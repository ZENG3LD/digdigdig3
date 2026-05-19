# BingX API Rate Limits

Research documentation for BingX API rate limiting system.

Last updated: 2026-01-21

## 1. REST API Limits

### 1.1 General Limits

BingX uses a **time-window based rate limiting system** (no weight system).

#### Historical Evolution

The rate limit system was upgraded in 2024-2025:

**Before April 2024:**
- All interfaces: 500 requests/minute per IP

**Current System (April 2024 onwards):**
- Separate limits for market data vs account interfaces
- IP-based limits
- Progressive increases to account interface limits

#### Current Limits (as of September 2025)

**Market Data Interfaces:**
- **100 requests per 10 seconds** per IP address
- Shared across all market data endpoints
- Public endpoints (no authentication required)

**Account Interfaces:**
- **2,000 requests per 10 seconds** per IP address (as of 2025-09-10)
- Both individual limits per UID AND total rate limit per IP
- Requires authentication

**Progressive Upgrade Timeline:**
- 2024-04-15: 150 requests/10s
- 2024-04-18: 300 requests/10s
- 2024-04-22: 600 requests/10s
- 2024-04-25: 1,000 requests/10s
- 2025-09-10: **2,000 requests/10s** (current)

### 1.2 Weight System

**BingX does NOT use a weight-based system** like Binance or other exchanges.

Instead, it uses **direct request counting**:
- Each API call counts as 1 request
- No endpoint-specific weights
- Simpler to implement and track

### 1.3 Endpoint-Specific Limits

#### Order Placement Endpoint

The order placement endpoint has a more restrictive limit:

**Endpoint:** `/openApi/swap/v2/trade/order`

**Limit:**
- **10 requests per second** (upgraded from 5 req/s on 2025-10-16)
- This is per API key or UID
- Independent of the general account interface limit

#### Public vs Private Endpoints

**Public Endpoints (Market Data):**
- Ticker, orderbook, klines, recent trades
- 100 requests/10s per IP
- No authentication required

**Private Endpoints (Account/Trading):**
- Balance, positions, orders, trades
- 2,000 requests/10s per IP
- Individual limits per UID
- Requires API key authentication

#### Spot vs Futures Limits

Based on the documentation:
- **Spot API:** Subject to the same market data (100/10s) and account (2,000/10s) limits
- **Perpetual Swap API:** Same general limits apply
- **Standard Contract API:** Same general limits apply

The limits are applied at the **interface type level** (market vs account), not per trading type.

## 2. Response Headers

### 2.1 Rate Limit Headers

**IMPORTANT:** BingX does NOT appear to return standard rate limit headers in API responses.

Unlike Binance (which uses `X-MBX-USED-WEIGHT-*`) or other exchanges that return:
- `X-RateLimit-Limit`
- `X-RateLimit-Remaining`
- `X-RateLimit-Reset`

BingX's API responses do NOT include these headers based on available documentation.

**Note:** The existing documentation in this file mentions these headers may be returned, but this was NOT confirmed in official BingX documentation. It's possible they exist, but they are not documented.

### 2.2 Authentication Header

For authenticated requests, BingX requires:

**Request Header:**
```
X-BX-APIKEY: <your_api_key>
```

### 2.3 Timestamp Requirement

- Request timestamp must be included as a parameter
- Server validates timestamp within 5000ms (5 seconds)
- Timestamp must be in milliseconds
- Requests with timestamps older than 5000ms are rejected

## 3. Error Handling

### 3.1 Rate Limit Error

**HTTP Status Code:** Not explicitly documented, but standard practice is **429 Too Many Requests**

**Error Code:** `100410`

**Error Message:** `FREQUENCY_LIMIT`

**JSON Response Format:**
```json
{
  "code": 100410,
  "msg": "FREQUENCY_LIMIT",
  "timestamp": 1732276055969
}
```

### 3.2 Other Common Error Codes

For reference, other BingX error codes:

**4XX Errors (Client-side):**
- `100001`: Signature authentication failed
- `100202`: Insufficient balance
- `100400`: Invalid parameter (e.g., `{"code":100400,"msg":"miss arguments","timestamp":1732276055969}`)
- `100440`: Order price deviates greatly from market price

**5XX Errors (Server-side):**
- Indicate problems on BingX's side

**General Format:**
- HTTP 200 indicates successful response
- Error responses include `code`, `msg`, and `timestamp` fields

### 3.3 Retry-After Header

**NOT DOCUMENTED** whether BingX returns a `Retry-After` header in 429 responses.

Based on available research, this header is likely **not included**.

## 4. Best Practices

### 4.1 Official Recommendations

From BingX documentation:

**Request Frequency:**
- "If requests are too frequent, the system will automatically limit the request"
- "It will automatically recover after a few minutes"

**WebSocket for Real-time Data:**
- For market data that updates frequently, use WebSocket instead of polling REST endpoints
- Reduces REST API pressure
- More efficient for real-time updates

### 4.2 Recovery Strategy

**When 429/FREQUENCY_LIMIT occurs:**

1. **Immediate action:** Stop sending requests
2. **Wait time:**
   - Minimum: Wait for current time window to expire (10 seconds for most endpoints)
   - Recommended: Wait "a few minutes" as per BingX docs
   - Conservative approach: 1-2 minutes before resuming
3. **Exponential backoff:**
   - First retry: 10 seconds
   - Second retry: 30 seconds
   - Third retry: 60 seconds
   - Fourth retry: 120 seconds

**Implementation approach:**
```rust
// Pseudocode
match response.status_code {
    429 => {
        // Wait for current 10s window + buffer
        sleep(Duration::from_secs(15));
        // Implement exponential backoff for retries
        retry_with_backoff();
    }
    _ => process_response()
}
```

### 4.3 Suggested Request Intervals

Since BingX doesn't provide `X-RateLimit-Remaining` headers, you must track limits client-side:

**Market Data (100 req/10s):**
- Max rate: 10 requests/second
- Safe rate: 8-9 requests/second (80-90% of limit)
- Minimum interval: 100ms between requests
- Recommended interval: 110-120ms

**Account Data (2,000 req/10s):**
- Max rate: 200 requests/second
- Safe rate: 160-180 requests/second (80-90% of limit)
- Minimum interval: 5ms between requests
- Recommended interval: 6-7ms

**Order Placement (10 req/s):**
- Max rate: 10 requests/second
- Safe rate: 8-9 requests/second
- Minimum interval: 100ms
- Recommended interval: 110-120ms

## 5. WebSocket Limits

### 5.1 Connection Limits

**Per IP:** Not explicitly documented

**Connection stability:**
- WebSocket connections can disconnect
- Implement automatic reconnection logic
- Monitor connection health with ping/pong

### 5.2 Subscription Limits Per Connection

**Spot Trading WebSocket:**
- **Maximum: 200 subscriptions per connection**
- Changed from unlimited (before August 20, 2024)
- Reason: Ensure stability with growing API user base

**Perpetual Futures WebSocket:**
- Subscription limits apply (specific number not documented)
- Likely similar to Spot (200 subscriptions)

### 5.3 Multiple Connections Strategy

If you need more than 200 subscriptions:
- Establish multiple WebSocket connections
- Distribute subscriptions across connections
- Each connection can handle up to 200 subscriptions

**Example:**
- 500 symbols to monitor
- Need 3 connections (500 / 200 = 2.5, round up to 3)
- Connection 1: 200 subscriptions
- Connection 2: 200 subscriptions
- Connection 3: 100 subscriptions

### 5.4 Message Rate Limits

**Not documented** in available sources.

Recommended approach:
- Monitor for disconnections or error messages
- If experiencing issues, reduce subscription count or message frequency

### 5.5 Heartbeat/Ping-Pong Mechanism

BingX WebSocket uses a **text-based ping-pong heartbeat** mechanism to maintain connections.

#### Message Format

**Server Ping:**
- The server sends a text message: `"Ping"`
- Messages are **gzip-compressed**, so you must decompress before checking

**Client Pong Response:**
- Client must respond with text message: `"Pong"`
- Send the response as plain text (not compressed)

#### Timing and Intervals

**Perpetual Futures WebSocket:**
- Server ping interval: **30 seconds** (changed from 5 seconds in July 2025)
- No-data timeout: **40 seconds** (changed from 10 seconds in July 2025)
- If client doesn't respond with "Pong" within timeout, server will disconnect

**Spot WebSocket:**
- Server ping interval: **5 seconds** (based on earlier documentation)
- Timeout: Not explicitly documented
- Recommended: respond to pings immediately

**Note:** The ping interval was updated in mid-2025 for Perpetual Futures to reduce message overhead.

#### Implementation Example

**JavaScript/Node.js:**
```javascript
const zlib = require('zlib');
const WebSocket = require('ws');

const ws = new WebSocket('wss://open-ws-swap.bingbon.pro/ws');

ws.on('message', (data) => {
    // BingX sends gzip-compressed messages
    const buffer = Buffer.from(data);
    const decodedMsg = zlib.gunzipSync(buffer).toString('utf-8');

    // Check if it's a ping message
    if (decodedMsg === 'Ping') {
        // Respond with pong
        ws.send('Pong');
        console.log('Received Ping, sent Pong');
        return;
    }

    // Process other messages
    const message = JSON.parse(decodedMsg);
    // ... handle market data
});
```

**Rust (pseudocode):**
```rust
use flate2::read::GzDecoder;
use std::io::Read;

fn handle_websocket_message(compressed_data: &[u8]) -> Result<(), Error> {
    // Decompress gzip data
    let mut decoder = GzDecoder::new(compressed_data);
    let mut decompressed = String::new();
    decoder.read_to_string(&mut decompressed)?;

    // Check for ping message
    if decompressed == "Ping" {
        // Send pong response
        websocket.send("Pong".into())?;
        return Ok(());
    }

    // Parse and process other messages
    let message: Message = serde_json::from_str(&decompressed)?;
    process_message(message)?;

    Ok(())
}
```

#### Connection Duration

**Maximum connection lifetime:**
- Not explicitly documented in official sources
- No mention of 24-hour disconnect like Binance
- Connections can persist indefinitely if:
  - Client responds to pings
  - Network remains stable
  - No server-side maintenance

**Reconnection requirements:**
- Implement automatic reconnection logic
- Reconnect on disconnect events
- Re-subscribe to all topics after reconnection
- Monitor for missed pings/pongs as a sign of connection issues

#### Key Implementation Points

1. **Decompression Required:**
   - ALL WebSocket messages from BingX are gzip-compressed
   - Must decompress before checking for "Ping"
   - Pong responses are sent uncompressed

2. **Response Time:**
   - Respond to pings immediately (within 40 seconds for Futures)
   - Don't queue pong responses behind other processing

3. **Connection Monitoring:**
   - Track time since last ping received
   - If no ping for extended period, connection may be stale
   - Implement reconnection logic

4. **Differences by Market:**
   - **Spot WebSocket:** 5-second ping interval (more frequent)
   - **Perpetual Futures WebSocket:** 30-second ping interval (less frequent)
   - Plan ping-pong handling accordingly

5. **Error Handling:**
   - If decompression fails, connection may be corrupted
   - If ping response fails to send, reconnect
   - Log all ping-pong activity for debugging

## 6. Implementation Notes

### 6.1 For Our RateLimiter

**Recommended Approach:**

Use a **token bucket algorithm** with time windows:

```rust
// Market data rate limiter
TokenBucket {
    capacity: 100,
    refill_rate: 100 tokens per 10 seconds (10 per second),
    window: 10 seconds
}

// Account data rate limiter
TokenBucket {
    capacity: 2000,
    refill_rate: 2000 tokens per 10 seconds (200 per second),
    window: 10 seconds
}

// Order placement rate limiter
TokenBucket {
    capacity: 10,
    refill_rate: 10 tokens per second,
    window: 1 second
}
```

### 6.2 Key Considerations

1. **No response headers:** Must track limits entirely client-side
2. **No weight system:** Simple 1:1 request counting
3. **Separate buckets:** Need different limiters for market vs account vs order endpoints
4. **IP-based limits:** All API keys from same IP share the limit
5. **Time window:** 10 second windows (not 1 minute like some exchanges)

### 6.3 Error Detection

Monitor for error code `100410` with message `FREQUENCY_LIMIT`:

```rust
if response.code == 100410 || response.msg == "FREQUENCY_LIMIT" {
    // Rate limit hit
    handle_rate_limit_error();
}
```

### 6.4 Client-Side Tracking

Since no headers are provided, implement request counting:

```rust
struct RateLimitTracker {
    requests: VecDeque<Instant>,  // Timestamps of recent requests
    window_size: Duration,         // 10 seconds
    max_requests: usize,           // 100 or 2000
}

impl RateLimitTracker {
    fn can_make_request(&mut self) -> bool {
        let now = Instant::now();
        // Remove requests outside the window
        self.requests.retain(|&timestamp| {
            now.duration_since(timestamp) < self.window_size
        });

        self.requests.len() < self.max_requests
    }

    fn record_request(&mut self) {
        self.requests.push_back(Instant::now());
    }
}
```

### 6.5 Endpoint Classification

Create an enum to classify endpoints:

```rust
enum RateLimitCategory {
    MarketData,      // 100/10s
    AccountData,     // 2000/10s
    OrderPlacement,  // 10/s
}

fn classify_endpoint(endpoint: &str) -> RateLimitCategory {
    if endpoint.contains("/trade/order") {
        RateLimitCategory::OrderPlacement
    } else if is_authenticated(endpoint) {
        RateLimitCategory::AccountData
    } else {
        RateLimitCategory::MarketData
    }
}
```

## 7. Comparison with Other Exchanges

| Feature | BingX | Binance | OKX |
|---------|-------|---------|-----|
| Rate limit type | Simple request counting | Weight-based | Weight-based |
| Response headers | None documented | Yes (`X-MBX-USED-WEIGHT-*`) | Yes |
| Time window | 10 seconds | 1 minute | 2 seconds |
| Market data limit | 100/10s | 1200 weight/minute | Varies |
| WebSocket sub limit | 200 per connection | 1024 per connection | 100-300 |
| WebSocket ping interval | 30s (Futures), 5s (Spot) | No server ping | Varies |
| Retry-After header | No | No | Yes |

**BingX Advantages:**
- Simpler rate limit logic (no weights to calculate)
- Higher account interface limit (2,000/10s = 200/s)

**BingX Disadvantages:**
- No response headers for tracking
- Less granular control (can't optimize heavy vs light requests)
- Must implement all tracking client-side

## 8. Sources

This research was compiled from the following sources:

- [BingX API Rate Limit Upgrade (Oct 2025)](https://bingx.com/en/support/articles/31103871611289)
- [BingX API Documentation](https://bingx-api.github.io/docs/)
- [BingX Standard Contract API Docs](https://github.com/BingX-API/BingX-Standard-Contract-doc/blob/main/REST%20API.md)
- [WebSocket Subscription Limits for Spot Trading](https://bingx.com/en/support/articles/36544879951641-adjustment-of-websocket-subscription-limits-for-spot-trading)
- [BingX.Net C# Library](https://github.com/JKorf/BingX.Net) - Implementation reference
- [BingX Swap API WebSocket Demo (GitHub Issue #6)](https://github.com/BingX-API/BingX-swap-api-doc/issues/6)
- [CCXT BingX Documentation](https://docs.ccxt.com/exchanges/bingx)
- BingX Rate Limit Upgrade Support Articles (2024-2025)
- [Understanding Ping Pong Frame WebSocket](https://www.videosdk.live/developer-hub/websocket/ping-pong-frame-websocket)

## 9. Testing Recommendations

To validate these limits:

1. **Create test script** that makes controlled requests
2. **Test market data limit:** Send 100 requests in 10 seconds, verify 101st fails
3. **Test account limit:** Send 2000 requests in 10 seconds, verify 2001st fails
4. **Test order limit:** Send 10 order requests in 1 second, verify 11th fails
5. **Capture error responses:** Verify error code 100410 and response format
6. **Check for headers:** Confirm no rate limit headers in responses
7. **Test recovery:** After hitting limit, wait and verify requests succeed again
8. **WebSocket limits:** Test subscribing to 200+ streams to confirm limit
9. **WebSocket ping-pong:** Verify gzip decompression and "Ping"/"Pong" message handling
10. **Ping interval:** Confirm 30-second interval for Futures, 5-second for Spot
11. **Timeout behavior:** Test what happens if client doesn't respond to ping within 40 seconds

## 10. Summary for Implementation

**Key takeaways for our connector:**

1. Implement **three separate rate limiters**:
   - Market data: 100 requests/10s
   - Account data: 2,000 requests/10s
   - Order placement: 10 requests/s

2. Use **client-side tracking** (no server headers available)

3. Implement **exponential backoff** on error code 100410

4. Use **token bucket algorithm** with 10-second windows

5. **Conservative limits:** Use 80-90% of max to avoid edge cases

6. **WebSocket strategy:** Use WebSocket for real-time data, limit to 200 subscriptions per connection

7. **WebSocket ping-pong:**
   - Decompress all incoming messages with gzip
   - Check for "Ping" text message
   - Respond immediately with "Pong"
   - Handle different intervals: 30s (Futures) vs 5s (Spot)

8. **Error handling:** Detect `FREQUENCY_LIMIT` and pause requests

9. **Recovery time:** Wait at least 10 seconds (one full window) after rate limit error

10. **Connection monitoring:** Track ping-pong timing, reconnect if connection stale

## 11. Known Limitations

Based on this research, the following information is **NOT available** in official documentation:

1. **Response headers:** Whether BingX returns `X-RateLimit-*` headers (not documented, but existing code suggests they might)
2. **Retry-After header:** Whether included in 429 responses
3. **Exact HTTP status code:** For rate limit errors (assumed 429, but not confirmed)
4. **WebSocket connection limits:** Maximum connections per IP
5. **Perpetual Futures WebSocket subscription limit:** Exact number (assumed 200 like Spot)
6. **WebSocket message rate limits:** Messages per second on WebSocket
7. **Maximum connection lifetime:** Whether there's a 24-hour limit like Binance
8. **Spot WebSocket ping timeout:** Exact timeout value (only Futures 40s is documented)

**Recommendation:** Test these aspects empirically to fill in the gaps.
