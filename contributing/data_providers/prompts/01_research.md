# Phase 1: Research Agent Prompt - Data Providers

## Agent Type
`research-agent`

## Variables
- `{PROVIDER}` - Provider name in lowercase (e.g., "polygon", "oanda", "coinglass")
- `{CATEGORY}` - Category (aggregators, forex, stocks, data_feeds)
- `{DOCS_URL}` - Official documentation URL

---

## Mission

**EXHAUSTIVE RESEARCH** - Document EVERYTHING this provider offers.

Unlike crypto exchanges (which are similar), data providers vary wildly.
Your job: Map out the ENTIRE API surface, not just standard endpoints.

---

## Output Folder

Create: `src/{CATEGORY}/{PROVIDER}/research/`

Examples:
- `src/stocks/us/polygon/research/`
- `src/forex/oanda/research/`
- `src/aggregators/defillama/research/`
- `src/data_feeds/coinglass/research/`

---

## Research Files (8 files)

═══════════════════════════════════════════════════════════════════════════════
FILE 1: api_overview.md
═══════════════════════════════════════════════════════════════════════════════

```markdown
# {PROVIDER} API Overview

## Provider Information
- Full name: ...
- Website: ...
- Documentation: {DOCS_URL}
- Category: {CATEGORY}

## API Type
- REST: Yes/No (base URL: ...)
- WebSocket: Yes/No (URL: ...)
- GraphQL: Yes/No (endpoint: ...)
- gRPC: Yes/No
- Other protocols: ...

## Base URLs
- Production: https://...
- Testnet/Sandbox: https://... (if exists)
- Regional endpoints: ... (if any)
- API version: v2 / v1 / etc.

## Documentation Quality
- Official docs: [URL]
- Quality rating: Excellent / Good / Adequate / Poor
- Code examples: Yes/No (languages: ...)
- OpenAPI/Swagger spec: Available? [URL if yes]
- SDKs available: Python, JavaScript, etc. (list)

## Licensing & Terms
- Free tier: Yes/No
- Paid tiers: Yes/No
- Commercial use: Allowed / Requires license
- Data redistribution: Allowed / Prohibited / Attribution required
- Terms of Service: [URL]

## Support Channels
- Email: ...
- Discord/Slack: ...
- GitHub: ...
- Status page: ...
```

═══════════════════════════════════════════════════════════════════════════════
FILE 2: endpoints_full.md
═══════════════════════════════════════════════════════════════════════════════

**CRITICAL:** Document EVERY endpoint, grouped by category.

**Don't skip anything** - even beta/experimental/paid-only endpoints.

```markdown
# {PROVIDER} - Complete Endpoint Reference

## Category: Standard Market Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v1/price | Current price | Yes | No | 60/min | Real-time |
| GET | /v1/ticker | 24h stats | Yes | No | 60/min | |
| GET | /v1/candles | OHLC bars | Yes | No | 60/min | Max 5000 bars |
| ... | ... | ... | ... | ... | ... | ... |

## Category: Historical Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/bars/minute | Minute bars | Paid | Yes | 300/min | Historical |
| ... | ... | ... | ... | ... | ... | ... |

## Category: Derivatives Analytics (if applicable)

Document endpoints for:
- Liquidations
- Open Interest
- Funding Rates
- Long/Short Ratios
- Options data (IV, Greeks, chains)

## Category: Fundamental Data (if applicable - stocks)

Document endpoints for:
- Company profiles
- Financial statements
- Earnings
- Dividends
- Analyst ratings
- Insider trading

## Category: On-chain Data (if applicable - crypto)

Document endpoints for:
- Wallet balances
- Transactions
- DEX trades
- Token transfers
- Smart contract events

## Category: Macro/Economic Data (if applicable)

Document endpoints for:
- Interest rates
- GDP
- Inflation metrics
- Employment data
- Economic calendars

## Category: Metadata

Document endpoints for:
- Symbol/instrument lists
- Exchange information
- Market hours
- Trading calendars
- Reference data

## Category: Account Management (if applicable)

Document endpoints for:
- API key info
- Usage/quota
- Subscription info
- Billing

## Category: WebSocket Control (if applicable)

Document endpoints for:
- Listen key generation (if needed)
- WebSocket authentication
- Connection management

## Parameters Reference

For complex endpoints, document all parameters:

### GET /v1/candles
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| symbol | string | Yes | - | Ticker symbol |
| from | timestamp | Yes | - | Start time (Unix ms) |
| to | timestamp | Yes | - | End time (Unix ms) |
| interval | string | No | "1h" | 1m,5m,15m,1h,1d |
| limit | int | No | 500 | Max 5000 |

... (для всех сложных endpoint'ов)
```

═══════════════════════════════════════════════════════════════════════════════
FILE 3: websocket_full.md
═══════════════════════════════════════════════════════════════════════════════

**If WebSocket NOT available:** Create file with "WebSocket: Not Available" and skip.

**If available:** Document EVERYTHING.

```markdown
# {PROVIDER} - WebSocket Documentation

## Availability: Yes / No

## Connection

### URLs
- Public streams: wss://...
- Private streams: wss://... (if separate)
- Regional: ... (if any)

### Connection Process
1. Connect to URL
2. Handshake: ...
3. Welcome message: ... (if any)
4. Auth: ... (if required)

## ALL Available Channels/Topics

**CRITICAL:** List EVERY channel, don't skip specialized ones.

| Channel/Topic | Type | Description | Auth? | Free? | Update Frequency | Example Subscription |
|---------------|------|-------------|-------|-------|------------------|---------------------|
| ticker | Public | Price updates | No | Yes | Real-time | {"type":"subscribe","channel":"ticker","symbol":"AAPL"} |
| trades | Public | Trade updates | No | Yes | Real-time | ... |
| orderbook | Public | L2 updates | No | Paid | Real-time | ... |
| liquidations | Public | Liq events | No | Yes | Real-time | ... (if applicable) |
| funding | Public | Funding rates | No | Yes | Every 8h | ... (if applicable) |
| klines | Public | OHLC updates | No | Yes | Per interval | ... |
| ... | ... | ... | ... | ... | ... | ... |

## Subscription Format

### Subscribe Message
```json
{
  "type": "subscribe",
  "channel": "ticker",
  "symbol": "AAPL"
}
```

### Unsubscribe Message
```json
{
  "type": "unsubscribe",
  "channel": "ticker",
  "symbol": "AAPL"
}
```

### Subscription Confirmation
```json
{
  "type": "subscribed",
  "channel": "ticker",
  "symbol": "AAPL"
}
```

## Message Formats (for EVERY channel)

### Ticker Update
```json
{
  "type": "ticker",
  "symbol": "AAPL",
  "price": 150.25,
  "bid": 150.24,
  "ask": 150.26,
  "volume": 1234567,
  "timestamp": 1234567890
}
```

### Trade Update
```json
{ ... }
```

### Orderbook Snapshot
```json
{ ... }
```

### Orderbook Delta/Update
```json
{ ... }
```

... (для КАЖДОГО channel'а)

## Heartbeat / Ping-Pong

**CRITICAL:** Document exactly!

### Who initiates?
- Server → Client ping: Yes/No
- Client → Server ping: Yes/No

### Message Format
- Binary ping/pong frames: Yes/No
- Text messages: "ping"/"pong" / other
- JSON messages: {"op":"ping"} / other

### Timing
- Ping interval: X seconds
- Timeout: X seconds (connection closed if no response)
- Client must send ping: Every X seconds (if required)

### Example
```
Server → Client: {"op":"ping","ts":1234567890}
Client → Server: {"op":"pong","ts":1234567890}
```

## Connection Limits
- Max connections per IP: X
- Max connections per API key: X
- Max subscriptions per connection: X
- Message rate limit: X messages/second
- Auto-disconnect after: X hours (if applicable)

## Authentication (for private channels)

If private channels exist:

### Method
- URL params: wss://...?apiKey=xxx
- Message after connect: {"op":"auth","key":"xxx"}
- Other: ...

### Auth Message Format
```json
{
  "op": "authenticate",
  "apiKey": "xxx",
  "signature": "yyy" (if required)
}
```

### Auth Success/Failure
```json
{
  "op": "auth_success"
}
```
```

═══════════════════════════════════════════════════════════════════════════════
FILE 4: authentication.md
═══════════════════════════════════════════════════════════════════════════════

```markdown
# {PROVIDER} - Authentication

## Public Endpoints

- Public endpoints exist: Yes/No
- Require authentication: Yes/No
- Rate limits without auth: X req/min

## API Key

### Required For
- All endpoints: Yes/No
- Paid tier only: Yes/No
- Rate limit increase: Yes/No
- Specific endpoints: (list)

### How to Obtain
- Sign up: [URL]
- API key management: [URL]
- Free tier includes key: Yes/No

### API Key Format
- Header: `X-API-Key: your_api_key_here`
- OR Query param: `?apiKey=xxx`
- OR Bearer token: `Authorization: Bearer xxx`
- OR Other: ...

### Multiple Keys
- Multiple keys allowed: Yes/No
- Rate limits per key: Yes/No
- Use cases for multiple keys: ...

## OAuth (if applicable)

### OAuth 2.0
- Supported: Yes/No
- Grant types: Authorization Code / Client Credentials / other
- Scopes: (list available scopes)
- Token endpoint: https://...
- Authorization endpoint: https://...

### Flow
1. Step 1: ...
2. Step 2: ...
3. ...

## Signature/HMAC (if applicable - rare for data providers)

**Usually NOT needed** - most data providers use simple API keys.

If signature required:

### Algorithm
- HMAC-SHA256 / HMAC-SHA512 / other

### Components
- Timestamp: Yes/No (format: Unix ms / seconds)
- Method: GET/POST
- Path: /v1/endpoint
- Query string: Yes/No
- Body: Yes/No (encoding: ...)

### Signature Construction
```
message = timestamp + method + path + query + body
signature = HMAC-SHA256(secret, message)
```

### Headers
```
X-API-Key: your_key
X-Signature: generated_signature
X-Timestamp: 1234567890
```

### Example
```
GET /v1/price?symbol=AAPL
Timestamp: 1234567890
Message: "1234567890GET/v1/price?symbol=AAPL"
Signature: HMAC-SHA256(secret, message) = "abc123..."
```

## Authentication Examples

### REST with API Key
```bash
curl -H "X-API-Key: your_key" https://api.example.com/v1/price?symbol=AAPL
```

### WebSocket with API Key
```javascript
const ws = new WebSocket('wss://ws.example.com?apiKey=your_key');
// OR
ws.send(JSON.stringify({op: 'auth', key: 'your_key'}));
```

## Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| 401 | Invalid API key | Check key is correct |
| 403 | Forbidden | Upgrade tier or check permissions |
| 429 | Rate limit | Wait or upgrade tier |
```

═══════════════════════════════════════════════════════════════════════════════
FILE 5: tiers_and_limits.md
═══════════════════════════════════════════════════════════════════════════════

**CRITICAL:** This is very important - impacts what we can do.

```markdown
# {PROVIDER} - Tiers, Pricing, and Rate Limits

## Free Tier

### Access Level
- Requires sign-up: Yes/No
- API key required: Yes/No
- Credit card required: No (hopefully)

### Rate Limits
- Requests per second: X
- Requests per minute: X
- Requests per hour: X (if applicable)
- Requests per day: X (if applicable)
- Burst allowed: Yes/No (X requests burst)

### Data Access
- Real-time data: Yes/No
- Delayed data: Yes/No (delay: 15min, 1h, etc.)
- Historical data: Yes/No (depth: X months/years)
- WebSocket: Allowed (Yes/No, limits: X connections)
- Data types: (list what's available)

### Limitations
- Symbols: Limited to X / Unlimited
- Endpoints: Some restricted / All available
- Features: (list restrictions)

## Paid Tiers

| Tier Name | Price | Rate Limit | Additional Features | WebSocket | Historical | Support |
|-----------|-------|------------|---------------------|-----------|------------|---------|
| Free | $0 | 60/min | Basic data | 1 conn | 1 year | Community |
| Starter | $29/mo | 300/min | + Real-time | 5 conn | 5 years | Email |
| Professional | $99/mo | 1000/min | + Extended data | Unlimited | Unlimited | Priority |
| Enterprise | Contact | Custom | Everything | Custom | Unlimited | Dedicated |

### Upgrade Benefits
- What new endpoints unlock?
- What data becomes available?
- Real-time vs delayed changes?
- Historical depth increases?

## Rate Limit Details

### How Measured
- Window: Per second / minute / hour
- Rolling window: Yes/No
- Fixed window: Yes/No

### Limit Scope
- Per IP address: Yes/No
- Per API key: Yes/No
- Per account: Yes/No
- Shared across: ... (if limits shared)

### Burst Handling
- Burst allowed: Yes/No
- Burst size: X requests
- Burst window: X seconds
- Token bucket: Yes/No

### Response Headers
```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1234567890
Retry-After: 30 (on 429 error)
```

### Error Response (HTTP 429)
```json
{
  "error": "Rate limit exceeded",
  "limit": 60,
  "remaining": 0,
  "reset": 1234567890,
  "retry_after": 30
}
```

### Handling Strategy
- Exponential backoff: Recommended
- Retry logic: ...
- Queue requests: ...

## Quota/Credits System (if applicable)

Some providers use credits instead of rate limits:

### How it Works
- Monthly quota: X,000 credits
- 1 request = Y credits (varies by endpoint)
- Overage: Blocked / Extra charges

### Credit Costs
| Endpoint Type | Credits per Request |
|---------------|---------------------|
| Price | 1 |
| Historical bars | 10 |
| Fundamentals | 50 |
| ... | ... |

## WebSocket Specific Limits

### Connection Limits
- Max connections per IP: X
- Max connections per API key: X
- Max connections total: X

### Subscription Limits
- Max subscriptions per connection: X
- Max symbols per subscription: X

### Message Rate Limits
- Messages per second: X
- Server may throttle: Yes/No
- Auto-disconnect on violation: Yes/No

### Connection Duration
- Max lifetime: 24 hours / Unlimited
- Auto-reconnect needed: Yes/No
- Idle timeout: X minutes (if applicable)

## Monitoring Usage

### Dashboard
- Usage dashboard: [URL if available]
- Real-time tracking: Yes/No
- Historical usage: Yes/No

### API Endpoints
- Check quota: GET /account/usage
- Check limits: GET /account/limits
- Response format: ...

### Alerts
- Email alerts: Yes/No (at X% usage)
- Webhook: Yes/No
```

═══════════════════════════════════════════════════════════════════════════════
FILE 6: data_types.md
═══════════════════════════════════════════════════════════════════════════════

**CRITICAL:** Catalog EVERYTHING this provider offers.

```markdown
# {PROVIDER} - Data Types Catalog

## Standard Market Data

- [x] Current Price
- [x] Bid/Ask Spread
- [x] 24h Ticker Stats (high, low, volume, change%)
- [x] OHLC/Candlesticks (intervals: 1m, 5m, 15m, 1h, 4h, 1d, etc.)
- [ ] Level 2 Orderbook (bids/asks depth)
- [x] Recent Trades
- [x] Volume (24h, intraday)

## Historical Data

- [x] Historical prices (depth: X years)
- [x] Minute bars (available: Yes/No)
- [x] Daily bars (depth: X years)
- [ ] Tick data (available: Yes/No)
- [x] Adjusted prices (splits, dividends)

## Derivatives Data (Crypto/Futures)

If applicable:

- [x] Open Interest (total, by exchange)
- [x] Funding Rates (current, historical)
- [x] Liquidations (real-time events)
- [x] Long/Short Ratios
- [x] Mark Price
- [x] Index Price
- [x] Basis (futures - spot spread)

## Options Data (if applicable)

- [x] Options Chains (strikes, expirations)
- [x] Implied Volatility
- [x] Greeks (delta, gamma, theta, vega)
- [x] Open Interest (per strike)
- [x] Historical option prices

## Fundamental Data (Stocks)

If applicable:

- [x] Company Profile (name, sector, industry, description)
- [x] Financial Statements (income, balance sheet, cash flow)
- [x] Earnings (EPS, revenue, guidance)
- [x] Dividends (history, yield)
- [x] Stock Splits
- [x] Analyst Ratings
- [x] Insider Trading
- [x] Institutional Holdings
- [x] Financial Ratios (P/E, P/B, ROE, debt/equity, etc.)
- [x] Valuation Metrics

## On-chain Data (Crypto)

If applicable:

- [x] Wallet Balances
- [x] Transaction History
- [x] DEX Trades (Uniswap, PancakeSwap, etc.)
- [x] Token Transfers (ERC-20, BEP-20)
- [x] Smart Contract Events
- [x] Gas Prices
- [x] Block Data
- [x] NFT Data

## Macro/Economic Data (Economics)

If applicable:

- [x] Interest Rates (Fed Funds, Treasury yields)
- [x] GDP (quarterly, annual)
- [x] Inflation (CPI, PPI, PCE)
- [x] Employment (NFP, unemployment rate, claims)
- [x] Retail Sales
- [x] Industrial Production
- [x] Consumer Confidence
- [x] PMI (Manufacturing, Services)
- [x] Economic Calendar (upcoming releases)

## Forex Specific

If applicable:

- [x] Currency Pairs (majors, minors, exotics)
- [x] Bid/Ask Spreads
- [x] Pip precision
- [x] Cross rates
- [x] Historical FX rates

## Metadata & Reference

- [x] Symbol/Instrument Lists
- [x] Exchange Information
- [x] Market Hours (regular, pre-market, after-hours)
- [x] Trading Calendars (holidays, half-days)
- [x] Timezone Info
- [x] Sector/Industry Classifications

## News & Sentiment (if applicable)

- [x] News Articles
- [x] Press Releases
- [x] Social Sentiment
- [x] Analyst Reports

## Unique/Custom Data

**What makes this provider special?**

Document any unique data this provider offers:
- Example: Coinglass has liquidation heatmaps
- Example: FRED has 800,000+ economic time series
- Example: Bitquery has DEX-specific on-chain analytics
```

═══════════════════════════════════════════════════════════════════════════════
FILE 7: response_formats.md
═══════════════════════════════════════════════════════════════════════════════

**EXACT JSON examples from official docs** - don't invent.

```markdown
# {PROVIDER} - Response Formats

## For EVERY important endpoint

### GET /v1/price
```json
{
  "symbol": "AAPL",
  "price": 150.25,
  "timestamp": 1234567890000
}
```

### GET /v1/ticker
```json
{
  "symbol": "AAPL",
  "last": 150.25,
  "bid": 150.24,
  "ask": 150.26,
  "high_24h": 152.50,
  "low_24h": 148.00,
  "volume_24h": 12345678,
  "change_24h": 2.50,
  "change_percent_24h": 1.69,
  "timestamp": 1234567890000
}
```

### GET /v1/candles
```json
[
  {
    "timestamp": 1234567890000,
    "open": 150.00,
    "high": 150.50,
    "low": 149.80,
    "close": 150.25,
    "volume": 1234567
  },
  ...
]
```

### GET /v1/liquidations (if applicable)
```json
{
  "exchange": "binance",
  "symbol": "BTCUSDT",
  "side": "long",
  "quantity": 1.5,
  "price": 45000.00,
  "value_usd": 67500,
  "timestamp": 1234567890000
}
```

... Document for EVERY endpoint category
```

═══════════════════════════════════════════════════════════════════════════════
FILE 8: coverage.md
═══════════════════════════════════════════════════════════════════════════════

```markdown
# {PROVIDER} - Data Coverage

## Geographic Coverage

### Regions Supported
- North America: Yes/No
- Europe: Yes/No
- Asia: Yes/No
- Other: ...

### Country-Specific
- US: Yes/No
- UK: Yes/No
- Japan: Yes/No
- India: Yes/No
- ... (list all)

### Restricted Regions
- Blocked countries: ...
- VPN detection: Yes/No
- Geo-fencing: Yes/No

## Markets/Exchanges Covered

### Stock Markets
- US: NYSE, NASDAQ, AMEX (Yes/No)
- UK: LSE (Yes/No)
- Japan: TSE (Yes/No)
- India: NSE, BSE (Yes/No)
- China: SSE, SZSE (Yes/No)
- ... (list all)

### Crypto Exchanges (if aggregator)
- Binance: Yes/No
- Coinbase: Yes/No
- Kraken: Yes/No
- ... (list ALL exchanges aggregated)

### Forex Brokers (if aggregator)
- ... (list)

### Futures/Options Exchanges
- CME: Yes/No
- CBOE: Yes/No
- ... (list)

## Instrument Coverage

### Stocks
- Total symbols: ~X,XXX
- US stocks: X,XXX
- International: X,XXX
- OTC: Yes/No
- Penny stocks: Yes/No

### Crypto
- Total coins: XXX
- Spot pairs: XXX
- Futures: XXX
- Perpetuals: XXX

### Forex
- Currency pairs: XX
- Majors: 7 pairs (Yes/No)
- Minors: ~XX pairs
- Exotics: ~XX pairs

### Commodities
- Metals: Gold, Silver, etc.
- Energy: Oil, Gas, etc.
- Agriculture: Corn, Wheat, etc.

### Indices
- US: S&P500, Nasdaq, Dow (Yes/No)
- International: FTSE, Nikkei, etc.
- Crypto: BTC Dominance, DeFi Index, etc.

## Data History

### Historical Depth
- Stocks: From year XXXX (X years)
- Crypto: From year XXXX (X years)
- Forex: From year XXXX (X years)

### Granularity Available
- Tick data: Yes/No
- 1-minute bars: Yes/No (from when?)
- 5-minute bars: Yes/No
- Hourly: Yes/No
- Daily: Yes/No (depth: X years)
- Weekly/Monthly: Yes/No

### Real-time vs Delayed
- Real-time: Yes/No (free tier?)
- Delayed: Yes/No (delay: 15min, 1h, EOD)
- Snapshot: Yes/No

## Update Frequency

### Real-time Streams
- Price updates: Every X ms
- Orderbook: Snapshot + delta / Full updates
- Trades: Real-time

### Scheduled Updates
- Fundamentals: Quarterly, Annual
- Economic data: Daily, Weekly, Monthly
- News: Real-time

## Data Quality

### Accuracy
- Source: Direct from exchange / Aggregated / Calculated
- Validation: Yes/No
- Corrections: Automatic / Manual

### Completeness
- Missing data: Common / Rare
- Gaps: How handled?
- Backfill: Available?

### Timeliness
- Latency: <X ms for real-time
- Delay: X seconds typical
- Market hours: Covered fully?
```

---

## Exit Criteria

- [x] All 8 research files created
- [x] Every file has EXACT data from official docs (no guessing)
- [x] All endpoints documented (including specialized ones)
- [x] All data types cataloged
- [x] Tier/pricing clearly documented
- [x] WebSocket documented (or noted as unavailable)
- [x] Coverage/limits understood

---

## Research Tips

1. **Use official docs ONLY** - don't invent examples
2. **Check for hidden endpoints** - some providers have undocumented beta features
3. **Test rate limits** - verify what docs claim
4. **Look for GraphQL/gRPC** - not just REST
5. **Check GitHub** - official SDKs often reveal more endpoints
6. **Search community forums** - Discord, Reddit for real usage
7. **Compare tiers** - understand what unlocks at each tier

---

## Special Cases

### If provider is GraphQL-based (e.g., Bitquery):
- Document GraphQL schema in `endpoints_full.md`
- Include example queries
- Note query complexity costs

### If provider is gRPC-based (e.g., Tinkoff):
- Document .proto file location
- List all RPC methods
- Include examples

### If provider has multiple products:
- Document each product separately
- Note which API keys work where
- Clarify data overlap

---

## After Research

**Do NOT start implementation yet.**

Research must be COMPLETE and REVIEWED before Phase 2.
