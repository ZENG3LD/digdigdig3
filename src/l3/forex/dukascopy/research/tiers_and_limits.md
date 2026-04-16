# Dukascopy - Tiers, Pricing, and Rate Limits

## Free Tier

### Access Level
- **Requires sign-up**: Yes (for SDK access) / No (for binary downloads)
- **API key required**: No (username/password for SDK)
- **Credit card required**: No (demo account is free)

### Signup Process

**Demo Account (Free)**:
1. Visit: https://www.dukascopy.com/swiss/english/forex/demo/
2. Fill registration form
3. Receive username/password via email
4. Extended validity available (no expiration for data access)

**Binary Downloads**:
- No signup needed
- Direct HTTP access to historical tick files

### Rate Limits

**Binary Downloads** (https://datafeed.dukascopy.com/):
- **Requests per second**: Undocumented (throttled after bulk downloads)
- **Requests per minute**: Undocumented
- **Requests per hour**: Undocumented
- **Requests per day**: Undocumented
- **Burst allowed**: Unknown
- **Behavior**: Connection throttling applied after excessive requests
- **Note**: Rate limiting introduced October 12, 2018

**JForex SDK (Demo)**:
- **Requests per second**: No explicit limit
- **Requests per minute**: No explicit limit
- **Fair use policy**: Applied (excessive usage may be restricted)
- **Concurrent connections**: 1 per account
- **Historical data**: Unlimited access via IHistory interface

**Third-Party REST** (unofficial):
- **Inherits JForex SDK limits**: Yes
- **Additional limits**: Depends on local server configuration

### Data Access

**Binary Downloads**:
- **Real-time data**: No (historical only)
- **Delayed data**: No
- **Historical data**: Yes (varies by instrument, often back to 2003+)
- **WebSocket**: No
- **Data types**: Tick data only (.bi5 files)

**JForex SDK (Demo)**:
- **Real-time data**: Yes (live market data)
- **Delayed data**: No (real-time only)
- **Historical data**: Yes (full depth available)
- **WebSocket**: No (Java listeners)
- **Data types**: Ticks, bars, custom feeds (renko, kagi, etc.)

### Limitations

**Binary Downloads**:
- **Symbols**: All available instruments
- **Endpoints**: Single endpoint pattern
- **Features**: Tick data only, hourly granularity
- **Format**: Compressed binary (.bi5), requires decompression
- **Time range**: Per-hour files only

**JForex SDK (Demo)**:
- **Symbols**: All 1,200+ instruments
- **Endpoints**: Full SDK functionality
- **Features**: Full historical and real-time access
- **Trading**: Demo trading only (no real money)
- **Strategies**: Can run automated strategies
- **Limitations**: Same data as live (no restrictions)

---

## Paid Tiers

### Live Trading Account

**Purpose**: Real money trading + API access

| Feature | Demo (Free) | Live Account |
|---------|-------------|--------------|
| Price | $0 | Varies (no minimum for basic) |
| Historical data | Full | Full |
| Real-time data | Yes | Yes |
| Trading | Demo only | Real money |
| API access | JForex SDK | JForex SDK |
| Minimum deposit | N/A | $0-100 (varies by region/type) |
| Support | Community | Email + phone |

**Account Types**:
- Standard: No minimum (varies by region)
- ECN: Higher liquidity
- Islamic: Swap-free accounts available

**Trading Costs**:
- Spreads: From 0.1 pips (ECN accounts)
- Commission: Varies by account type and volume
- No API access fees

### FIX API Access

**Purpose**: Professional/institutional trading

| Feature | Value |
|---------|-------|
| Price | Included with account |
| Minimum deposit | **USD 100,000** |
| Real-time data | Yes (via FIX) |
| Historical data | Yes (via JForex SDK) |
| Trading | Yes (via FIX) |
| Support | Dedicated support |
| Rate limits | See below |

**Requirements**:
- Live account with $100,000+ balance
- IP address registration
- FIX 4.4 client implementation
- Time synchronization (GMT)

**Additional Features**:
- Direct market access
- Custom trading algorithms
- Co-location available (contact Dukascopy)
- Multiple connections (trading + data feed)

### Media/Portal API Access

**Purpose**: Data embedding for websites/portals

| Feature | Value |
|---------|-------|
| Price | Free (with advertising placement) |
| Application | Required |
| Data types | Real-time quotes, historical data |
| Format | JSON, XML (widget-based) |
| Usage | Public display only |
| Redistribution | Not allowed |

**Application Process**:
1. Visit: https://www.dukascopy.com/trading-tools/api/apply
2. Describe use case
3. Provide advertising space
4. Receive API access

---

## Rate Limit Details

### Binary Downloads

**How Measured**:
- Window: Unknown (likely per-hour or per-day)
- Rolling window: Unknown
- Fixed window: Unknown

**Limit Scope**:
- Per IP address: Yes
- Per API key: N/A (no keys)
- Shared across: All requests from same IP

**Behavior**:
- Gradual throttling after bulk downloads
- Connection speed reduction
- Temporary blocks possible

**No Response Headers**:
- X-RateLimit-Limit: Not provided
- X-RateLimit-Remaining: Not provided
- X-RateLimit-Reset: Not provided
- Retry-After: Not provided

**Error Response**:
- HTTP 429: Possible (not confirmed)
- HTTP 503: Service temporarily unavailable

### JForex SDK

**How Measured**:
- Fair use policy (no hard limits published)
- Server-side monitoring

**Limit Scope**:
- Per account: Yes
- Concurrent sessions: 1 per account
- Instruments: No limit (subscribe to all if needed)

**Data Access Limits**:
- IHistory.getTicks(): No explicit limit on number of ticks
- IHistory.getBars(): No explicit limit on number of bars
- Real-time subscriptions: No explicit limit on instruments

**Practical Limits**:
- Memory: Loading large datasets may consume significant memory
- Network: Large historical queries may take time
- Processing: Client-side processing limits apply

### FIX API

**How Measured**:
- Per-connection, per-account
- Fixed windows (per second, per minute)

**Hard Limits**:

| Limit Type | Value | Scope |
|-----------|-------|-------|
| Max orders per second | 16 | Per account |
| Max open positions | 100 | Per account |
| Connection attempts | 5 per minute | Per server, per IP |
| Heartbeat interval | 30 seconds (default) | Per connection |
| Session timeout | 2 hours | Without restoration |
| Message rate | No explicit limit | Fair use |

**Burst Handling**:
- Burst allowed: No (hard limit at 16 orders/sec)
- Exceeding limit: Orders rejected
- Rejection reason: Rate limit (tag 58)

**Error Response** (ExecutionReport with OrdRejReason):
```
8=FIX.4.4|35=8|39=8|150=8|103=99|
58=Rate limit exceeded|10=XXX|
```

---

## Quota/Credits System

**Not Applicable**: Dukascopy does not use a credits/quota system.

All limits are:
- Time-based (requests per second/minute)
- Connection-based (max connections)
- Fair use policy (for demo accounts)

---

## WebSocket Specific Limits

**Official WebSocket**: Not available

**Third-Party WebSocket** (unofficial):

### Connection Limits
- Max connections per IP: Not specified (single-user deployment)
- Max connections per account: Inherits JForex SDK limit (1 session)
- Max connections total: Depends on server resources

### Subscription Limits
- Max subscriptions per connection: No explicit limit
- Max symbols per subscription: Limited by URL parameter length
- Instruments: All available via JForex SDK

### Message Rate Limits
- Messages per second: Driven by JForex tick rate (100-500ms per update)
- Server throttling: Possible (inherits SDK throttling)
- Auto-disconnect on violation: No

### Connection Duration
- Max lifetime: Unlimited (persistent)
- Auto-reconnect needed: Yes (if SDK session lost)
- Idle timeout: No

---

## Monitoring Usage

### Dashboard

**Binary Downloads**:
- Usage dashboard: Not available
- Real-time tracking: No
- Historical usage: No
- Recommendation: Monitor via server logs

**JForex SDK**:
- Usage dashboard: Not available
- Account statistics: Available in JForex platform
- API usage: No specific metrics
- Historical queries: No tracking

**FIX API**:
- Usage dashboard: Not available
- Order stats: Available via FIX messages
- Rate limit status: Not exposed
- Monitoring: Client-side tracking recommended

### API Endpoints

**No usage tracking endpoints** available in any API.

### Alerts

- Email alerts: No
- Webhook: No
- Recommendation: Implement client-side monitoring

---

## Handling Strategy

### Binary Downloads

**Rate Limit Detection**:
```python
import time
import requests

def download_with_retry(url, max_retries=3):
    for attempt in range(max_retries):
        response = requests.get(url, timeout=30)

        if response.status_code == 200:
            return response.content
        elif response.status_code == 429:
            wait_time = 2 ** attempt  # Exponential backoff
            print(f"Rate limited, waiting {wait_time}s")
            time.sleep(wait_time)
        elif response.status_code == 503:
            time.sleep(60)  # Server busy
        else:
            raise Exception(f"HTTP {response.status_code}")

    raise Exception("Max retries exceeded")
```

**Best Practices**:
- Delay between requests: 100-500ms recommended
- Exponential backoff on errors
- Download during off-peak hours
- Cache downloaded files
- Respect rate limits

### JForex SDK

**Memory Management**:
```java
// Load large datasets in chunks
long chunkSize = 7 * 24 * 60 * 60 * 1000; // 1 week
long start = startTime;

while (start < endTime) {
    long end = Math.min(start + chunkSize, endTime);
    List<IBar> bars = history.getBars(instrument, period, side, start, end);

    // Process bars
    processBars(bars);

    // Clear memory
    bars.clear();
    start = end;
}
```

**Async Loading**:
```java
// Use async loading for large datasets
history.readBars(instrument, period, side, from, to,
    new LoadingDataListener() {
        @Override
        public void newData(ITimedData data) {
            IBar bar = (IBar) data;
            processBar(bar);
        }

        @Override
        public void loadingFinished(boolean allLoaded, long start, long end, long currentTime) {
            System.out.println("Loading complete: " + allLoaded);
        }
    },
    new LoadingProgressListener() {
        @Override
        public void dataLoaded(long start, long end, long currentTime, String information) {
            System.out.println("Progress: " + information);
        }

        @Override
        public void loadingFinished(boolean allLoaded, long start, long end, long currentTime) {
            System.out.println("Finished");
        }
    }
);
```

### FIX API

**Order Rate Limiting**:
```python
import time
from collections import deque

class OrderRateLimiter:
    def __init__(self, max_per_second=16):
        self.max_per_second = max_per_second
        self.orders = deque()

    def can_send_order(self):
        now = time.time()
        # Remove orders older than 1 second
        while self.orders and self.orders[0] < now - 1.0:
            self.orders.popleft()

        return len(self.orders) < self.max_per_second

    def send_order(self, order):
        if not self.can_send_order():
            time.sleep(0.1)  # Wait 100ms
            return self.send_order(order)

        # Send order
        result = send_fix_order(order)
        self.orders.append(time.time())
        return result
```

---

## Cost Comparison

| Access Method | Setup Cost | Monthly Cost | Data Cost | Trading Cost |
|---------------|------------|--------------|-----------|--------------|
| Binary Downloads | $0 | $0 | $0 | N/A |
| Demo SDK | $0 | $0 | $0 | Demo only |
| Live SDK (Basic) | $0-100 | $0 | $0 | Spreads + commission |
| FIX API | $100,000 | $0 | $0 | Spreads + commission |
| Media API | $0 | Advertising | $0 | N/A |

**Notes**:
- No API access fees (unlike many data providers)
- No per-request charges
- No monthly subscription for API
- Costs only apply to live trading (spreads/commissions)

---

## Comparison with Other Providers

| Provider | Free Tier | Historical Ticks | Real-time | API Type | Rate Limit |
|----------|-----------|------------------|-----------|----------|------------|
| Dukascopy | Yes | Yes (unlimited) | Yes | SDK/FIX | Fair use |
| Polygon | Yes | Limited | Limited | REST | 5 req/min |
| Alpha Vantage | Yes | No | Yes | REST | 25 req/day |
| IEX Cloud | Yes | No | Yes | REST | 50,000 msgs/mo |
| Quandl | Yes | Limited | No | REST | 50 req/day |

**Dukascopy Advantages**:
- Free unlimited historical tick data
- No hard rate limits on free tier
- Professional-grade data quality
- Swiss bank reliability

**Dukascopy Disadvantages**:
- No official REST API
- Requires Java SDK or FIX client
- Rate limits not documented
- Commercial use requires agreement

---

## Summary

### Free Access
- **Binary downloads**: Unlimited historical ticks (with throttling)
- **Demo SDK**: Full API access, unlimited data
- **Best for**: Backtesting, research, education

### Paid Access
- **Live account**: Real trading, same API access as demo
- **FIX API**: Professional traders, $100k minimum
- **Best for**: Production trading, algorithmic trading

### Key Points
- No API fees (only trading costs)
- Rate limits not documented (fair use policy)
- Free tier extremely generous
- Commercial use requires agreement
- No credit/quota system
