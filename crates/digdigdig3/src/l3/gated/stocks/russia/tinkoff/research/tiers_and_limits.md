# Tinkoff Invest API - Tiers, Pricing, and Rate Limits

## Free Tier

### Access Level
- **Requires sign-up**: Yes (Tinkoff Investments brokerage account required)
- **API key required**: Yes (token generation in account settings)
- **Credit card required**: No (but brokerage account may require identity verification)

### Rate Limits
- **Requests per second**: Dynamic (varies by trading activity)
- **Requests per minute**: Dynamic (varies by trading activity)
- **Requests per hour**: Not specified (dynamic system)
- **Requests per day**: Not specified (dynamic system)
- **Burst allowed**: Not documented (likely yes, within dynamic limits)

**Important**: Tinkoff uses dynamic rate limiting, NOT fixed limits. Active traders get higher allowances.

### Data Access
- **Real-time data**: Yes (all market data is real-time)
- **Delayed data**: No (no delayed feeds - everything is real-time or not available)
- **Historical data**: Yes (depth: up to 10 years, from 1970-01-01 for some instruments)
- **WebSocket**: Yes (unlimited connections within rate limits)
- **Data types**: Full access to all data types (market data, fundamentals, etc.)

### Limitations
- **Symbols**: Unlimited (access to all instruments available on platform)
- **Endpoints**: All available (no endpoint restrictions)
- **Features**: Full API access for free (no paid tiers for API itself)

**Trading limitations**:
- Orders over 6,000,000 RUB require additional confirmation (not available via API)
- Some instruments require qualified investor status
- Some instruments may be forbidden for API trading

## Paid Tiers

**NO PAID TIERS FOR API ACCESS**

The Tinkoff Invest API is completely free for all Tinkoff Investments clients.

### Pricing Model
- **API access**: Free
- **Market data**: Free
- **Real-time streaming**: Free
- **Historical data**: Free
- **Trading via API**: Free

### Brokerage Fees (separate from API)
Tinkoff charges standard brokerage commissions on trades:
- **Commission structure**: Varies by tariff plan
- **Tariff info**: Use `GetUserTariff` API method
- **Plans**: Different commission rates based on trading volume
- **API to check**: `UsersService.GetUserTariff()`

**Important**: API is free, but trading commissions apply like manual trading.

## Rate Limit Details

### How Measured
- **Window**: Per minute (primary)
- **Rolling window**: Likely yes (not explicitly documented)
- **Fixed window**: Not documented
- **Dynamic adjustment**: Yes (based on trading activity)

### Limit Scope
- **Per IP address**: Not the primary factor
- **Per API key**: Yes (each token has independent limits)
- **Per account**: Likely yes (trading activity considered)
- **Shared across**: Not applicable (no fixed shared limits)

### Dynamic Rate Limiting System

**How it works**:
1. **Trading activity tracking**: System monitors your trading fees/volume
2. **Limit adjustment**: More trading = higher API rate limits
3. **Automatic scaling**: Limits scale dynamically without user action

**Activity levels**:
- **High-volume traders**: Very high limits (hundreds or thousands of requests/min)
- **Medium-volume traders**: Standard-high limits
- **Low-volume traders**: Standard limits (sufficient for most use cases)
- **No trading**: Minimum limits (enough for monitoring and research)

**Platform capacity**: 20,000 requests/second peak (shared across all users)

### Burst Handling
- **Burst allowed**: Not explicitly documented (likely yes)
- **Burst size**: Dynamic
- **Burst window**: Not specified
- **Token bucket**: Likely (standard for gRPC rate limiting)

### Response Headers

**gRPC Metadata** (not HTTP headers):
```
x-tracking-id: unique-request-id-12345
```

**Rate limit headers**: NOT provided (unlike REST APIs)
- No `X-RateLimit-Limit` header
- No `X-RateLimit-Remaining` header
- No `X-RateLimit-Reset` header

**Why no headers**:
- gRPC uses different error model
- Dynamic limits make static headers less useful
- Errors provide feedback when limits exceeded

### Error Response (gRPC RESOURCE_EXHAUSTED)

**Error code**: 80002
**gRPC Status**: RESOURCE_EXHAUSTED
**Message**: Request rate exceeded

**Example error**:
```json
{
  "code": 80002,
  "message": "Request rate exceeded per minute quota",
  "details": "Too many requests. Please reduce request rate."
}
```

**Other rate limit errors**:
- **80001**: Concurrent stream limit exceeded (too many WebSocket/gRPC streams)
- **80003**: SMS transmission quota exhausted (for operations requiring SMS confirmation)

### Handling Strategy

**Recommended approach**:
1. **Exponential backoff**:
   ```
   delay = min(max_delay, base_delay * 2^retry_count)
   Example: 1s, 2s, 4s, 8s, 16s, 32s (max)
   ```

2. **Retry logic**:
   - Detect 80001, 80002, 80003 errors
   - Implement backoff with jitter
   - Maximum retries: 5-10

3. **Request queuing**:
   - Queue requests locally
   - Process at sustainable rate
   - Adaptive rate adjustment based on errors

4. **Circuit breaker**:
   - Stop requests after repeated failures
   - Cool-down period before retry
   - Prevents cascade failures

**Example (pseudo-code)**:
```python
def call_api_with_retry(request, max_retries=5):
    for retry in range(max_retries):
        try:
            return client.call(request)
        except ResourceExhausted as e:
            if e.code == 80002:
                delay = min(32, 2 ** retry)
                time.sleep(delay + random.uniform(0, 1))
            else:
                raise
    raise Exception("Max retries exceeded")
```

## Quota/Credits System (if applicable)

**NOT APPLICABLE**

Tinkoff Invest API does NOT use quota/credits system. Rate limiting is dynamic and transparent.

## WebSocket Specific Limits

### Connection Limits
- **Max connections per IP**: Dynamic (based on activity)
- **Max connections per API key**: Dynamic
- **Max connections total**: Not specified (high limit)

**Error**: 80001 (Concurrent stream limit exceeded)

### Subscription Limits
- **Max subscriptions per connection**: Not explicitly documented (very high)
- **Max symbols per subscription**: Not specified (batch subscriptions supported)

**Recommendation**: Use reasonable subscription counts (hundreds, not thousands per connection)

### Message Rate Limits
- **Messages per second**: Dynamic (part of overall rate limiting)
- **Server may throttle**: Yes (via rate limit errors)
- **Auto-disconnect on violation**: Possible (connection closed on persistent violations)

### Connection Duration
- **Max lifetime**: Unlimited (no automatic disconnect)
- **Auto-reconnect needed**: No (persistent connections)
- **Idle timeout**: Not specified (ping/pong keeps alive)

**Best practice**: Implement reconnection logic for network issues, not for limits.

## Monitoring Usage

### Dashboard
- **Usage dashboard**: Not publicly available
- **Real-time tracking**: No user-facing dashboard
- **Historical usage**: Not provided to users

### API Endpoints
- **Check quota**: Not applicable (no fixed quota)
- **Check limits**: Not available (dynamic limits not exposed)
- **Response format**: N/A

**Workaround**: Monitor error rates (80001, 80002) to gauge proximity to limits

### Alerts
- **Email alerts**: No (no usage alerts from Tinkoff)
- **Webhook**: No
- **Rate limit warnings**: None (only errors when exceeded)

**Recommendation**: Implement application-level monitoring:
- Track request count per minute
- Monitor error rates
- Alert on repeated 80002 errors
- Dashboard for request rate trends

## Increasing Rate Limits

### Organic Method (Recommended)
1. **Increase trading activity**: More trades = higher limits
2. **Higher trading fees**: More revenue for Tinkoff = better limits
3. **Automatic adjustment**: System adjusts limits without manual request

### Explicit Request
- **Contact**: al.a.volkov@tinkoff.ru (for public software developers)
- **Use case**: Building public trading platform/tool
- **Benefits**: Dedicated app name, potential limit increase, technical support
- **Requirement**: Describe project and expected usage

## Comparison with Other Brokers

| Feature | Tinkoff | Interactive Brokers | Alpaca |
|---------|---------|---------------------|--------|
| **API Pricing** | Free | Free | Free |
| **Rate Limits** | Dynamic | Fixed (60/sec) | Fixed (200/min) |
| **Limit Increase** | Auto (trading) | Manual request | Paid tiers |
| **Real-time Data** | Free | Free (delayed) | Paid |
| **WebSocket** | Free | Free | Free |

## Rate Limit Best Practices

### Design Principles
1. **Assume limits exist**: Even without documented numbers
2. **Implement backoff**: Always use exponential backoff
3. **Cache data**: Reduce redundant requests
4. **Batch requests**: When API supports it
5. **Stream when possible**: WebSocket more efficient than polling

### Caching Strategies
- **Instrument data**: Cache for 1 hour (rarely changes)
- **Trading schedules**: Cache for 1 day
- **Account info**: Cache for 5 minutes
- **Market data**: Don't cache (use WebSocket streaming)

### Optimization Tips
1. **Use WebSocket for real-time data**: More efficient than polling
2. **Batch instrument queries**: Request multiple at once
3. **Request only needed intervals**: Don't fetch all candle timeframes
4. **Filter operations**: Use date/instrument filters in GetOperations
5. **Pagination**: Use GetOperationsByCursor for large datasets

### Anti-Patterns (Avoid These)
- ❌ Polling market data every second (use WebSocket)
- ❌ Fetching full portfolio on every tick (use PortfolioStream)
- ❌ Requesting all instruments repeatedly (cache reference data)
- ❌ No retry logic (temporary failures will break application)
- ❌ Ignoring error codes (80001, 80002 indicate limit issues)

## Sandbox Rate Limits

- **Same as production**: Sandbox uses same rate limiting system
- **Separate limits**: Sandbox token has independent limits from production
- **Testing limitations**: Can test rate limit handling in sandbox

## Special Considerations

### High-Frequency Trading (HFT)
- **Supported**: Yes, but within rate limits
- **Dynamic limits help**: Active HFT gets higher limits
- **Recommendation**: Start with reasonable frequency, scale up as limits increase

### Market Data Collection
- **Bulk historical data**: Use GetCandles with max ranges
- **Real-time collection**: Use WebSocket streams
- **Rate consideration**: Fetching years of data may hit limits
- **Solution**: Spread requests over time, implement queuing

### Multi-Account Trading
- **Account-specific tokens**: Each has independent limits
- **Total limit**: Sum of all token limits (per account activity)
- **Recommendation**: Use separate tokens per account for isolation

## Error Code Reference (Rate Limiting)

| Code | gRPC Status | Description | Action |
|------|-------------|-------------|--------|
| 80001 | RESOURCE_EXHAUSTED | Concurrent stream limit exceeded | Close unused connections |
| 80002 | RESOURCE_EXHAUSTED | Request rate exceeded per minute | Exponential backoff, reduce rate |
| 80003 | RESOURCE_EXHAUSTED | SMS transmission quota exhausted | Wait for quota reset, avoid SMS-required operations |

## Summary

### Key Takeaways
1. ✅ **API is completely free** (no paid tiers)
2. ✅ **Dynamic rate limiting** (not fixed numbers)
3. ✅ **Trading activity = higher limits** (automatic scaling)
4. ✅ **All data is real-time** (no delayed feeds)
5. ✅ **No quota system** (simple rate limiting)
6. ⚠️ **No limit visibility** (no dashboard or headers)
7. ⚠️ **Must implement backoff** (handle 80002 errors)
8. ⚠️ **Increase via trading** (or contact for public software)

### Recommended Rate Strategy
- **Start conservatively**: Assume ~100 req/min baseline
- **Monitor errors**: Track 80002 occurrences
- **Implement backoff**: Exponential with jitter
- **Use WebSocket**: For real-time data (more efficient)
- **Cache reference data**: Reduce redundant requests
- **Scale gradually**: Increase rate as error-free
- **Trade actively**: To unlock higher limits automatically

### Contact for Support
- **Email**: al.a.volkov@tinkoff.ru
- **Purpose**: Public software registration, technical support
- **Benefits**: Dedicated app name, potential limit discussion
