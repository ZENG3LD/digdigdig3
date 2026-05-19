# Futu OpenAPI - Tiers, Pricing, and Rate Limits

## Free Tier

### Access Level
- Requires sign-up: **Yes** (Futubull/moomoo account required)
- API key required: **No** (uses account authentication via OpenD)
- Credit card required: **No** (basic account opening free, but some markets require deposits)

### Account Requirements
- **Account Opening**: Must open account via Futubull/moomoo app (free)
- **API Access**: Complete API compliance questionnaire (free)
- **OpenD**: Download and install OpenD gateway (free)
- **Market Access**: Depends on account type and market permissions

### Rate Limits (Basic Account)

#### REST-like Endpoints (Request/Response)
- **Standard Limit**: **60 requests per 30 seconds** for most endpoints
- **Burst allowed**: Yes (can send 60 requests immediately, then wait 30s)
- **Window**: Rolling 30-second window

#### Trading Endpoints (Order Operations)
- **Trading Limit**: **15 requests per 30 seconds** per account
- **Applies to**:
  - `place_order()`
  - `modify_order()`
  - `cancel_order()`
  - `cancel_all_order()`
- **Minimum gap**: **0.02 seconds** between consecutive order requests (20ms)
- **Per-account**: Limit is per trading account, not per connection

#### High-Frequency Data (After Subscription)
- **No request limit** for:
  - `get_stock_quote()`
  - `get_rt_ticker()`
  - `get_cur_kline()`
  - `get_order_book()`
  - `get_broker_queue()`
  - `get_rt_data()`
- **Condition**: Must subscribe to data first
- **Updates**: Server-push driven (no polling needed)

### Data Access (Free Tier)

#### Real-time Data
- **Real-time**: Yes (depends on market and quote level)
- **Delayed data**: Not applicable (either real-time or no access)

#### Historical Data
- **Historical candlesticks**: Yes (quota-limited)
- **Depth**: Up to **20 years** of daily candlesticks
- **Intraday**: Yes (1m, 5m, 15m, 30m, 1h intervals available)

#### WebSocket (Subscription Push)
- **Allowed**: Yes
- **Connections**: Multiple contexts allowed (reasonable use)
- **Subscriptions**: Quota-limited (see below)

#### Data Types Available
- **Market Data**: Basic quotes, candlesticks, trades, order book (LV1)
- **Trading**: Full trading capabilities (place, modify, cancel orders)
- **Account**: Account info, positions, orders, fills
- **Paper Trading**: Simulated trading with same API

### Subscription Quotas (CRITICAL)

| Account Tier | Real-time Subscription Quota | Historical K-line Quota (30 days) | Based On |
|-------------|------------------------------|----------------------------------|----------|
| **Basic** | **100** | **100** | New account, no deposits |
| **Standard** | **300** | **300** | Assets > 10,000 HKD **OR** Monthly trading volume > 500,000 HKD **OR** >100 filled orders/month |
| **High Volume** | **1,000** | **1,000** | Assets > 100,000 HKD **OR** Monthly trading volume > 2,000,000 HKD **OR** >300 filled orders/month |
| **Premium** | **2,000** | **2,000** | Assets > 500,000 HKD **OR** Monthly trading volume > 10,000,000 HKD **OR** >1000 filled orders/month |

**How Quotas Work:**

1. **Real-time Subscription Quota**:
   - Each security code + SubType = 1 quota
   - Example: `US.AAPL` with `[QUOTE, TICKER, K_1M]` = **3 quotas used**
   - Example: `['US.AAPL', 'HK.00700']` with `[QUOTE]` = **2 quotas used**
   - Quota consumed when subscribed
   - Quota freed when unsubscribed (min 1-minute wait before unsubscribe)

2. **Historical K-line Quota**:
   - Each unique security's historical candlestick request = 1 quota
   - "Repeated requests for historical candlestick of the same stock within the last 30 days will not be counted repeatedly"
   - Quota resets after **30 days** from first request for that security
   - Example: Request `US.AAPL` daily bars = 1 quota used (lasts 30 days)
   - Example: Request `US.AAPL` again within 30 days = 0 additional quota

3. **Automatic Tier Upgrade**:
   - Tiers automatically activate within **2 hours** of meeting criteria
   - Based on total assets across all accounts (HK + US + A-share combined)
   - Based on trading volume or fill count (whichever criterion met first)

### Special Quota Limits
- **SF (Singapore/Japan Futures) Users**: Limited to **50 securities** maximum for subscription, regardless of quota tier

### Limitations (Basic Account)

#### Symbols
- **Symbols**: Unlimited (any supported market, subject to quote rights)
- **Restriction**: Quote rights determine access, not free tier

#### Endpoints
- **Some restricted**: No (all endpoints available)
- **Feature gated**: By quote rights and trading permissions, not tier

#### Features
- **Paper Trading**: Unlimited (simulated accounts free)
- **Live Trading**: Requires account funding and market permissions
- **Order Types**: All order types available (Limit, Market, Stop, Trailing, etc.)
- **Advanced Orders**: Algo orders (TWAP, VWAP) query-only, cannot place

## Paid Tiers (Quote Subscriptions)

**Note**: API usage itself is free. Paid tiers are for **market data subscriptions**, not API access.

### Quote Subscription Cards

| Market | Subscription Type | Approx Price | What It Unlocks | Required For |
|--------|------------------|--------------|-----------------|--------------|
| **HK Stocks (LV2)** | Hong Kong LV2 Quote | ~$10-20/month | 40-level depth, Broker queue | Broker queue data |
| **US Nasdaq TotalView** | Nasdaq TotalView | ~$30-50/month | 60-level order book | Deep US market depth |
| **A-Shares** | A-Share LV1 | Free (mainland China only) | Basic quotes | A-share access for non-mainland |
| **HK Futures** | Hong Kong Futures | Varies | HK futures quotes | Futures data |
| **US Futures (CME)** | CME Data | Varies | CME Group futures | US futures data |
| **US Options** | US Options | Free (if qualified) | Options chains, Greeks | US options data |

### Quote Level Comparison

| Feature | HK LV1 (Free) | HK LV2 (Paid) | US Basic (Free) | US TotalView (Paid) |
|---------|--------------|---------------|----------------|-------------------|
| **Price** | Free | ~$10-20/mo | Free | ~$30-50/mo |
| **Quote Delay** | Real-time | Real-time | Real-time | Real-time |
| **Order Book** | 10 levels | 40 levels | 10 levels | 60 levels |
| **Broker Queue** | No | Yes | No | No |
| **Tick Data** | Yes | Yes | Yes | Yes |
| **Candlesticks** | Yes | Yes | Yes | Yes |
| **SubType.BROKER** | No | Yes | N/A | N/A |

### US Options Free Access (Special)

**Eligibility**:
- Total account assets > **$3,000 USD**
- Have trading history (previous trades)

**If Qualified**:
- US options data free
- Options chains, Greeks, IV
- No subscription card needed

### How to Purchase

1. **Via Futubull/moomoo App**:
   - Open app → Market → Data Subscription
   - Select desired markets
   - Choose subscription period (monthly/yearly)
   - Payment via account balance or credit card

2. **Auto-apply to API**:
   - Subscriptions purchased in app automatically apply to OpenD
   - No separate API subscription
   - Takes effect immediately (or within minutes)

3. **Manage Subscriptions**:
   - App → Me → Settings → Data Subscription → Manage

## Rate Limit Details

### How Measured
- **Window**: Per 30 seconds (rolling window)
- **Rolling window**: Yes (not fixed window reset)
- **Fixed window**: No

Example:
- 10:00:00 - Send 60 requests
- 10:00:15 - Cannot send (still in 30s window)
- 10:00:30 - Window resets, can send 60 more

### Limit Scope
- **Per IP address**: No (limits are per connection/account)
- **Per API key**: N/A (no API keys)
- **Per account**: Yes (trading limits are per account)
- **Per connection**: Implicit (connection represents account)

### Burst Handling
- **Burst allowed**: Yes (can send max requests immediately)
- **Burst size**: 60 requests (standard) or 15 requests (trading)
- **Burst window**: 30 seconds
- **Token bucket**: Not explicitly, but similar behavior

### Response Headers
**Not applicable** - Futu uses custom TCP protocol, not HTTP.

Rate limit info not provided in response headers. Client must track requests.

### Error Response (Rate Limit Exceeded)

```python
ret, data = quote_ctx.some_method()

# Rate limited:
# ret = RET_ERROR (-1)
# data = "freq limit" or "frequency limitation"
```

**No structured error code** - must check error message string.

### Handling Strategy

**Recommended Approach**:

1. **Client-side Throttling**:
   ```python
   import time
   from collections import deque

   class RateLimiter:
       def __init__(self, max_requests=60, window=30):
           self.max_requests = max_requests
           self.window = window
           self.requests = deque()

       def wait_if_needed(self):
           now = time.time()
           # Remove requests outside window
           while self.requests and self.requests[0] < now - self.window:
               self.requests.popleft()

           # Check if limit reached
           if len(self.requests) >= self.max_requests:
               sleep_time = self.requests[0] + self.window - now
               if sleep_time > 0:
                   time.sleep(sleep_time)
               self.requests.popleft()

           self.requests.append(now)

   # Usage
   limiter = RateLimiter(max_requests=60, window=30)

   for i in range(100):
       limiter.wait_if_needed()
       ret, data = quote_ctx.get_market_snapshot(['US.AAPL'])
   ```

2. **Exponential Backoff** (on error):
   ```python
   import time

   def call_with_retry(func, max_retries=3, base_delay=1):
       for attempt in range(max_retries):
           ret, data = func()
           if ret == RET_OK:
               return ret, data

           if "freq limit" in data.lower():
               delay = base_delay * (2 ** attempt)
               print(f"Rate limited, waiting {delay}s...")
               time.sleep(delay)
           else:
               # Other error, don't retry
               return ret, data

       return RET_ERROR, "Max retries exceeded"
   ```

3. **Queue Requests**:
   - For high-volume applications, queue requests
   - Worker thread processes queue at safe rate
   - Prevents rate limit errors

### Trading Specific Limits

**Order Operations**:
- **Limit**: 15 requests per 30 seconds **per account**
- **Minimum gap**: 0.02 seconds (20ms) between consecutive order operations

**Example**:
```python
import time

def place_order_safe(trade_ctx, **order_params):
    # Ensure minimum 20ms gap
    time.sleep(0.02)

    ret, data = trade_ctx.place_order(**order_params)

    if ret != RET_OK and "freq limit" in data.lower():
        # Wait and retry once
        time.sleep(2)  # Wait 2 seconds
        ret, data = trade_ctx.place_order(**order_params)

    return ret, data
```

**Best Practice**:
- Don't send order bursts (spread out over time)
- Use modify_order instead of cancel+place when possible
- Batch operations where possible
- Use order types that don't require frequent modification (e.g., GTD instead of constant re-placing)

## Subscription Quota Management

### Checking Current Usage

```python
# Check current subscriptions
ret, data = quote_ctx.query_subscription()
if ret == RET_OK:
    print(data)
    # Shows all active subscriptions and quota usage
```

**Response Format**:
```python
# DataFrame with columns:
# code, subtype_list, total_used
# e.g., 'US.AAPL', '[QUOTE, TICKER, K_1M]', 3
```

### Quota Management Strategy

1. **Subscribe Only What's Needed**:
   ```python
   # BAD - subscribes to all types
   quote_ctx.subscribe(['US.AAPL'], [SubType.QUOTE, SubType.TICKER,
                       SubType.K_1M, SubType.ORDER_BOOK, SubType.RT_DATA])
   # Uses 5 quotas for one security!

   # GOOD - subscribe only necessary
   quote_ctx.subscribe(['US.AAPL'], [SubType.QUOTE])
   # Uses 1 quota
   ```

2. **Unsubscribe Unused**:
   ```python
   # Free up quota by unsubscribing
   quote_ctx.unsubscribe(['OLD.SECURITY'], [SubType.QUOTE])

   # Wait at least 1 minute before re-subscribing
   time.sleep(60)

   # Subscribe to new security
   quote_ctx.subscribe(['NEW.SECURITY'], [SubType.QUOTE])
   ```

3. **Rotate Subscriptions** (for large watchlists):
   ```python
   # If monitoring > quota limit securities
   watchlist = ['US.AAPL', 'US.GOOGL', 'US.MSFT', ...]  # 500 securities
   quota = 100  # Basic tier

   # Monitor in batches
   for i in range(0, len(watchlist), quota):
       batch = watchlist[i:i+quota]
       quote_ctx.subscribe(batch, [SubType.QUOTE])

       # Monitor for 5 minutes
       time.sleep(300)

       # Unsubscribe and wait
       quote_ctx.unsubscribe(batch, [SubType.QUOTE])
       time.sleep(60)  # Wait 1 minute
   ```

4. **Prioritize Securities**:
   - Keep high-priority securities always subscribed
   - Rotate low-priority securities

### Historical K-line Quota

**Query Quota Status**:
```python
ret, data = quote_ctx.request_history_kl_quota()
if ret == RET_OK:
    print(data)
    # Shows used/remaining quota and details
```

**Response Format**:
```python
# DataFrame with columns:
# code, request_time, request_count
# Shows when each security's historical data was requested
```

**Optimization**:
- Cache historical data locally (data doesn't change for past bars)
- Request once per security per 30 days
- Use `get_cur_kline()` (subscription-based, no quota) for recent data
- Only use `request_history_kline()` for initial historical load

## Connection Limits

### Max Connections
- **Per IP**: Not explicitly limited (reasonable use)
- **Per account**: Not explicitly limited (reasonable use)
- **Concurrent contexts**: Multiple allowed (e.g., QuoteContext + TradeContext simultaneously)

### OpenD Limits
- **OpenD connections**: Single OpenD can serve multiple client connections
- **Recommended**: One OpenD per trading account for isolation

### Connection Duration
- **Max lifetime**: Unlimited (persistent TCP connection)
- **Idle timeout**: None
- **Auto-disconnect**: None (connection persists until closed)

## Monitoring Usage

### Dashboard
- **Usage dashboard**: Via Futubull/moomoo app
  - App → Me → Settings → API Usage
- **Real-time tracking**: Limited (no live API usage metrics)
- **Historical usage**: No detailed API call history

### API Endpoints
- **Check quota**: `query_subscription()` for subscription quota
- **Check quota**: `request_history_kl_quota()` for historical K-line quota
- **No general usage API**: No endpoint to check total request count

### Alerts
- **Email alerts**: Not available
- **Webhook**: Not available
- **Client-side**: Must implement own monitoring

### Self-Monitoring

```python
import time
from collections import Counter

class APIMonitor:
    def __init__(self):
        self.calls = Counter()
        self.errors = Counter()
        self.start_time = time.time()

    def log_call(self, method_name, ret_code, error_msg=""):
        self.calls[method_name] += 1
        if ret_code != RET_OK:
            self.errors[f"{method_name}: {error_msg}"] += 1

    def report(self):
        elapsed = time.time() - self.start_time
        print(f"\n=== API Usage Report ({elapsed:.1f}s) ===")
        print(f"Total calls: {sum(self.calls.values())}")
        print(f"\nBy method:")
        for method, count in self.calls.most_common():
            print(f"  {method}: {count}")
        print(f"\nErrors ({sum(self.errors.values())} total):")
        for error, count in self.errors.most_common():
            print(f"  {error}: {count}")

# Usage
monitor = APIMonitor()

ret, data = quote_ctx.get_stock_quote(['US.AAPL'])
monitor.log_call('get_stock_quote', ret, data if ret != RET_OK else "")

# ... more API calls ...

monitor.report()
```

## Cost Summary

### Free Forever
- **API Access**: Free
- **Basic Account**: Free
- **Paper Trading**: Free
- **Basic Quote Rights**: Free (market-dependent)
- **Trading Commissions**: Same as app (no extra OpenAPI fee)

### Paid (Optional)
- **Quote Subscriptions**: $10-50/month (if advanced data needed)
- **Account Funding**: Required for live trading (min deposit varies by market)

### No Hidden Fees
- **No per-request charges**: Unlimited API requests (within rate limits)
- **No data volume charges**: No charge based on data received
- **No connection fees**: No charge for maintaining connections

## Recommendations

### For Basic Tier (100 quota)
- Subscribe to max **33 securities** with 3 SubTypes each (QUOTE, TICKER, K_1M)
- Or **100 securities** with QUOTE only
- Suitable for: Small watchlist, single-security strategies, backtesting

### For Standard Tier (300 quota)
- Subscribe to **100 securities** with 3 SubTypes each
- Or **300 securities** with QUOTE only
- Suitable for: Medium watchlist, multi-security strategies, sector monitoring

### For High Volume Tier (1,000 quota)
- Subscribe to **333 securities** with 3 SubTypes each
- Or **1,000 securities** with QUOTE only
- Suitable for: Large watchlist, market scanning, quantitative strategies

### For Premium Tier (2,000 quota)
- Subscribe to **666 securities** with 3 SubTypes each
- Or **2,000 securities** with QUOTE only
- Suitable for: Full market coverage, institutional-level strategies

### Upgrading Tier
**Fastest way to upgrade**:
1. Deposit funds to reach asset threshold (fastest)
2. Active trading to reach volume threshold (slower)
3. Combination of both

**Example**: Deposit 10,000 HKD (~$1,280 USD) to instantly upgrade from 100 to 300 quota

## Limits Compared to Competitors

| Feature | Futu | Interactive Brokers | Alpaca | Binance |
|---------|------|-------------------|--------|---------|
| **Request Rate** | 60/30s | 50/s | 200/min | 1200/min |
| **Trade Rate** | 15/30s | No stated limit | 200/min | 10/s |
| **Subscription Quota** | 100-2,000 | Unlimited | Unlimited | Unlimited |
| **Historical Quota** | 100-2,000/30d | Unlimited | Unlimited | Unlimited |
| **API Cost** | Free | Free | Free | Free |
| **Data Cost** | $0-50/mo | $0-105/mo | Free | Free |

**Futu's Strengths**:
- No API cost (some brokers charge)
- Generous real-time access after subscription
- Clear quota system (predictable)

**Futu's Weaknesses**:
- Subscription quota system (limited watchlist size for basic)
- Historical K-line quota (must manage carefully)
- Lower request rate than some crypto exchanges
