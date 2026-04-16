# Bitfinex API v2 Rate Limits

## REST API Rate Limits

### General Rate Limit Policy

Bitfinex implements a **per-IP-address** rate limiting system:
- Rate limits vary by endpoint from **10 to 90 requests per minute**
- When exceeded, the IP is **blocked for one full minute**
- Different endpoints have different limits
- **No rate limit headers provided** - must track client-side

### Authenticated Endpoints

**Most authenticated endpoints**: **90 requests per minute**

This includes:
- `/auth/w/order/submit` - Submit order
- `/auth/w/order/cancel` - Cancel order
- `/auth/w/order/cancel/multi` - Cancel multiple orders
- `/auth/w/order/update` - Update order
- `/auth/r/orders` - Retrieve orders
- `/auth/r/orders/hist` - Order history
- `/auth/r/trades/hist` - Trade history
- `/auth/r/wallets` - Wallets
- `/auth/r/positions` - Positions
- All other authenticated endpoints

### Public Endpoints

**Rate limits**: **10-90 requests per minute** (varies by endpoint)

Common public endpoints:
- `/ticker/{symbol}` - Individual ticker
- `/tickers` - Multiple tickers
- `/book/{symbol}/{precision}` - Order book
- `/trades/{symbol}/hist` - Trade history
- `/candles/{candle}/hist` - Candle data
- `/platform/status` - Platform status
- `/conf/*` - Configuration endpoints

Exact limits per endpoint are not publicly documented, but generally:
- High-frequency data (ticker, book): Lower limits (~10-30/min)
- Historical data (trades, candles): Higher limits (~60-90/min)

### Order-Specific Rate Limits

**Base Limit**: **1,000 orders per 5 minutes** per user

**Important Details**:
- Shared across **all API connections** for the same account
- Applied per user account, not per IP
- Increases based on trading volume

**Volume-Based Scaling Formula**:
```
limit = 1000 + (TOTAL_PAIRS_PLATFORM * 60 * 5) / (250000000 / USER_VOL_LAST_30d)
```

Where:
- `TOTAL_PAIRS_PLATFORM`: Number of trading pairs (~101 currently)
- `USER_VOL_LAST_30d`: User's 30-day trading volume in USD

**Example Calculation**:
For a user with $10,000,000 in 30-day volume:
```
1000 + (101 * 300) / (250000000 / 10000000)
= 1000 + 30300 / 25
= 1000 + 1212
= 2,212 orders per 5 minutes
```

Higher volume traders get proportionally higher limits.

## Rate Limit Headers

**Status**: Bitfinex does **NOT** provide standard rate limit headers.

**Missing Headers**:
- `X-RateLimit-Limit` - Not provided
- `X-RateLimit-Remaining` - Not provided
- `X-RateLimit-Reset` - Not provided
- `Retry-After` - Not provided

**Implication**: You **must** implement client-side rate tracking. There's no way to query your remaining quota before hitting the limit.

## Rate Limit Error Response

When rate limit is exceeded:

```json
{
  "error": "ERR_RATE_LIMIT"
}
```

HTTP Status Code: **429 Too Many Requests**

**Penalty**: IP address is blocked for **60 seconds**

**Complete Error Example**:
```
HTTP/1.1 429 Too Many Requests
Content-Type: application/json

{"error": "ERR_RATE_LIMIT"}
```

## WebSocket Connection Limits

### Authenticated Connections

**Limit**: **5 connections per 15 seconds**

**Domain**: `wss://api.bitfinex.com/ws/2`

**Penalty**: 15-second rate limit if exceeded

**Details**:
- Applies to authenticated WebSocket connections
- Per IP address
- If limit exceeded, must wait 15 seconds before retry
- Each connection reserves 1 channel for account info

### Public Connections

**Limit**: **20 connections per minute**

**Domain**: `wss://api-pub.bitfinex.com/ws/2`

**Penalty**: 60-second rate limit if exceeded

**Details**:
- Applies to public WebSocket connections
- Per IP address
- If limit exceeded, must wait 60 seconds before retry

### WebSocket Error Response

When exceeding connection limits:

```
HTTP/1.1 429 Too Many Requests
Error: unexpected server response (429)
```

The WebSocket handshake fails with HTTP 429 status code.

### Channel Subscription Limits

**Per Connection**: **25-30 subscriptions maximum**

Official documentation states both:
- "You can subscribe to 25 channels at the same time on the same connection" (public docs)
- "All websocket connections have a limit of 30 subscriptions to public market data feed channels" (general docs)

**Conservative approach**: Use **25 as the safe limit**.

**Channel Types**:
- Public market data: tickers, books, candles, trades, status
- Authenticated channels: account info (reserved on auth connections, uses chanId 0)

**Error on Exceeding Limit**:
```json
{
  "event": "error",
  "msg": "Reached limit of open channels",
  "code": 10305
}
```

### WebSocket Error Codes

All WebSocket errors follow this format:
```json
{
  "event": "error",
  "msg": "ERROR_MESSAGE",
  "code": ERROR_CODE
}
```

**Subscription Error Codes**:
- `10000`: Unknown event
- `10001`: Unknown pair
- `10300`: Subscription failed (generic)
- `10301`: Already subscribed
- `10302`: Unknown channel
- **`10305`: Reached limit of open channels** (subscription limit exceeded)
- `10400`: Unsubscribe failed (generic)
- `10401`: Not subscribed

**Critical**: Code `10305` indicates you've hit the subscription limit.

### WebSocket Message Rate Limits

**Not Documented**: Bitfinex does not publish specific limits on:
- Messages per second per connection
- Order submission rate via WebSocket
- Update frequency limits

No known message-level throttling beyond connection and subscription limits.

## Nonce Constraints

The nonce used in authentication has limits:

**Maximum Value**: `9007199254740991` (JavaScript's MAX_SAFE_INTEGER)

**Requirements**:
- Must be strictly increasing
- Cannot be reused
- Cannot exceed the maximum value

**Impact on Rate Limits**:
- If using multiple authenticated connections from same IP, use separate API keys
- Each API key maintains independent nonce tracking
- Prevents nonce conflicts and authentication errors

## Best Practices

### Rate Limit Management

1. **Implement Client-Side Request Tracking**

   Since Bitfinex doesn't provide rate limit headers, you **must** track requests locally:

   ```rust
   use std::collections::VecDeque;
   use std::time::{Duration, Instant};

   pub struct RateLimiter {
       requests: VecDeque<Instant>,
       max_requests: u32,
       window: Duration,
   }

   impl RateLimiter {
       pub fn new(max_requests: u32, window: Duration) -> Self {
           Self {
               requests: VecDeque::new(),
               max_requests,
               window,
           }
       }

       pub async fn acquire(&mut self) {
           // Remove old requests outside window
           let cutoff = Instant::now() - self.window;
           while self.requests.front().map_or(false, |&t| t < cutoff) {
               self.requests.pop_front();
           }

           // Wait if at limit
           if self.requests.len() >= self.max_requests as usize {
               let oldest = self.requests[0];
               let wait_until = oldest + self.window;
               let now = Instant::now();

               if wait_until > now {
                   tokio::time::sleep(wait_until - now).await;
               }

               self.requests.pop_front();
           }

           // Record this request
           self.requests.push_back(Instant::now());
       }
   }

   // Usage for different rate limits:
   let rest_limiter = RateLimiter::new(10, Duration::from_secs(60));      // Conservative
   let order_limiter = RateLimiter::new(1000, Duration::from_secs(300));  // Orders
   let ws_auth = RateLimiter::new(5, Duration::from_secs(15));            // WS auth
   let ws_public = RateLimiter::new(20, Duration::from_secs(60));         // WS public
   ```

2. **Use WebSocket for Real-Time Data**
   - REST API for one-time queries
   - WebSocket for continuous updates
   - Reduces REST API calls significantly

3. **Batch Requests Where Possible**
   - Use `/tickers` for multiple symbols instead of individual `/ticker/{symbol}` calls
   - Use `/auth/w/order/cancel/multi` for canceling multiple orders

4. **Implement Exponential Backoff**
   ```rust
   async fn retry_with_backoff<F, T>(mut request: F) -> Result<T>
   where
       F: FnMut() -> Pin<Box<dyn Future<Output = Result<T>> + Send>>,
   {
       let mut delay = Duration::from_secs(60);

       loop {
           match request().await {
               Ok(result) => return Ok(result),
               Err(e) if is_rate_limit_error(&e) => {
                   warn!("Rate limit hit, waiting {} seconds", delay.as_secs());
                   tokio::time::sleep(delay).await;

                   // Add jitter to avoid thundering herd
                   let jitter = rand::thread_rng().gen_range(1..10);
                   tokio::time::sleep(Duration::from_secs(jitter)).await;

                   // Don't increase delay - rate limit block is fixed at 60s
                   continue;
               }
               Err(e) => return Err(e),
           }
       }
   }
   ```

5. **Track Order Rate Separately**
   ```rust
   pub struct OrderRateLimiter {
       orders: VecDeque<Instant>,
       max_orders: u32,  // Default 1000, adjust based on volume
       window: Duration, // 5 minutes
   }

   impl OrderRateLimiter {
       pub fn can_submit_order(&mut self) -> bool {
           let cutoff = Instant::now() - self.window;
           self.orders.retain(|&t| t > cutoff);
           self.orders.len() < self.max_orders as usize
       }

       pub fn record_order(&mut self) {
           self.orders.push_back(Instant::now());
       }
   }
   ```

6. **Separate API Keys for Multiple Clients**
   - Prevents nonce conflicts
   - Better rate limit distribution
   - Easier to track usage per client

### WebSocket Best Practices

1. **Connection Management**
   - Maintain persistent connections
   - Don't reconnect on every request
   - Implement automatic reconnection with backoff

2. **Avoid Rapid Reconnections**

   **Critical**: "If your WS connection is dropped, please make sure to not make multiple reconnects in short succession"

   ```rust
   struct ReconnectManager {
       last_connect: Instant,
       min_interval: Duration,
       backoff: ExponentialBackoff,
   }

   impl ReconnectManager {
       fn can_reconnect(&self) -> bool {
           Instant::now().duration_since(self.last_connect) >= self.min_interval
       }

       async fn reconnect(&mut self) -> Result<Connection> {
           while !self.can_reconnect() {
               tokio::time::sleep(Duration::from_millis(100)).await;
           }

           let delay = self.backoff.next_backoff();
           tokio::time::sleep(delay).await;

           self.last_connect = Instant::now();
           // ... connection logic
       }
   }
   ```

3. **Optimize Channel Usage**
   - Subscribe to multiple symbols on fewer connections
   - Use appropriate precision levels (P0-P4) for order books
   - Unsubscribe from unused channels

4. **Monitor Connection Health**
   - Respond to heartbeat messages (sent every 15 seconds)
   - Implement ping/pong mechanism
   - Detect and handle connection drops

5. **Handle Subscription Limit (10305)**
   ```rust
   pub struct SubscriptionManager {
       connections: Vec<Connection>,
       subscriptions: HashMap<ConnectionId, Vec<Channel>>,
       max_per_connection: usize, // 25
   }

   impl SubscriptionManager {
       pub async fn subscribe(&mut self, channel: Channel) -> Result<()> {
           // Find connection with available slots
           for (conn_id, subs) in &self.subscriptions {
               if subs.len() < self.max_per_connection {
                   match self.send_subscribe(conn_id, &channel).await {
                       Ok(_) => {
                           self.subscriptions.get_mut(conn_id)
                               .unwrap()
                               .push(channel);
                           return Ok(());
                       }
                       Err(e) if e.code() == 10305 => {
                           // Hit limit unexpectedly, try next connection
                           continue;
                       }
                       Err(e) => return Err(e),
                   }
               }
           }

           // Need new connection
           let new_conn = self.create_connection().await?;
           self.subscriptions.insert(new_conn.id(), vec![channel]);
           Ok(())
       }

       fn handle_error_10305(&mut self, conn_id: ConnectionId) {
           warn!("Connection {} hit subscription limit (10305)", conn_id);
           // Create new connection for future subscriptions
       }
   }
   ```

## Monitoring and Logging

### Track Rate Limit Usage

```rust
struct RateLimitMonitor {
    requests_per_minute: HashMap<String, usize>,
    last_reset: Instant,
}

impl RateLimitMonitor {
    fn log_request(&mut self, endpoint: &str) {
        *self.requests_per_minute
            .entry(endpoint.to_string())
            .or_insert(0) += 1;

        if Instant::now().duration_since(self.last_reset) >= Duration::from_secs(60) {
            self.print_stats();
            self.requests_per_minute.clear();
            self.last_reset = Instant::now();
        }
    }

    fn print_stats(&self) {
        for (endpoint, count) in &self.requests_per_minute {
            if *count > 60 {
                warn!("{}: {} requests/min (approaching limit)", endpoint, count);
            } else {
                info!("{}: {} requests/min", endpoint, count);
            }
        }
    }
}
```

### Handle Rate Limit Errors

```rust
fn is_rate_limit_error(status: StatusCode, body: &str) -> bool {
    if status == StatusCode::TOO_MANY_REQUESTS {
        return true;
    }

    // Also check body for ERR_RATE_LIMIT
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(error) = json.get("error") {
            if error == "ERR_RATE_LIMIT" {
                return true;
            }
        }
    }

    false
}

fn is_subscription_limit_error(code: u32) -> bool {
    code == 10305
}

async fn handle_api_error(error: ApiError) -> Result<()> {
    match error {
        ApiError::RateLimit => {
            warn!("Rate limit exceeded, waiting 60 seconds");
            tokio::time::sleep(Duration::from_secs(60)).await;
            Ok(())
        }
        ApiError::WebSocketError { code: 10305, .. } => {
            warn!("Subscription limit reached, creating new connection");
            // Handle by creating new connection
            Ok(())
        }
        ApiError::Auth => {
            error!("Authentication error, check API keys");
            Err(error)
        }
        _ => Err(error),
    }
}
```

## Rate Limit Distribution Strategy

### Option 1: Single Connection (Simple)
- Use one REST API client
- Track requests manually
- Stay well under 90 req/min limit (e.g., 60 req/min safe margin)

### Option 2: Multiple API Keys (Advanced)
- Create separate API keys for different functions
- Trading operations: Key A
- Market data: Key B
- Account queries: Key C
- Each gets independent 90 req/min limit

### Option 3: Hybrid REST + WebSocket (Recommended)
- REST for:
  - Initial data fetch
  - Order submission/cancellation
  - Historical data retrieval
- WebSocket for:
  - Real-time price updates
  - Order book streaming
  - Trade notifications
  - Account updates

## Comparison: REST vs WebSocket Limits

| Aspect | REST API | WebSocket |
|--------|----------|-----------|
| Rate Limit | 10-90 req/min | N/A (connection-based) |
| Connections | Unlimited | 5 auth/15s, 20 pub/min |
| Subscriptions | N/A | 25 channels per connection |
| Data Updates | On request | Real-time push |
| Latency | Higher | Lower |
| Use Case | One-time queries | Continuous updates |
| Overhead | Per request | Per connection |
| Headers Provided | No | No |

## Conservative Rate Limit Recommendations

For safe operation without hitting limits:

**REST API**:
- Unknown endpoints: Start at **10 req/min**
- Public market data: **30 req/min**
- Private authenticated: **60 req/min** (leave 30 req/min margin)
- Orders: **800 orders/5min** (leave 200 order margin)

**WebSocket**:
- Authenticated: **Max 4 connections per 15s** (leave 1 connection margin)
- Public: **Max 15 connections per minute** (leave 5 connection margin)
- Subscriptions: **Max 20 per connection** (leave 5 channel margin)

**Gradual Testing**:
1. Start with conservative limits
2. Monitor for 429 errors
3. Gradually increase if no errors occur
4. Document learned limits for each endpoint

## Testing Rate Limits

### Safe Testing Approach

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limit_handling() {
        let client = BitfinexClient::new();
        let mut limiter = RateLimiter::new(10, Duration::from_secs(60));

        for i in 0..100 {
            limiter.acquire().await;

            match client.get_ticker("tBTCUSD").await {
                Ok(_) => println!("Request {} succeeded", i),
                Err(e) if is_rate_limit_error(&e) => {
                    panic!("Hit rate limit despite limiter! Request {}", i);
                }
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        }
    }

    #[tokio::test]
    async fn test_subscription_limit() {
        let mut manager = SubscriptionManager::new();

        // Try subscribing to 30 channels on one connection
        for i in 0..30 {
            let channel = Channel::Ticker(format!("tBTC{}", i));
            match manager.subscribe(channel).await {
                Ok(_) if i < 25 => println!("Subscription {} ok", i),
                Err(e) if e.code() == 10305 && i >= 25 => {
                    println!("Hit subscription limit at {}", i);
                }
                result => panic!("Unexpected result at {}: {:?}", i, result),
            }
        }
    }
}
```

**Warning**: Don't test rate limits aggressively on production API. Use testnet or implement gradual testing.

## Emergency Rate Limit Recovery

If you've hit rate limits:

1. **Immediate Stop**: Halt all requests/connections immediately
2. **Wait for Cooldown**:
   - REST: 60 seconds minimum
   - WebSocket auth: 15 seconds minimum
   - WebSocket public: 60 seconds minimum
3. **Add Jitter**: Wait an additional random 1-10 seconds to avoid synchronization
4. **Check System Clock**: Ensure correct time for nonce generation
5. **Review Logs**: Identify which endpoints/connections caused issues
6. **Implement Rate Limiting**: Add throttling before retry
7. **Gradual Resume**: Don't resume at full rate
   - Start at 50% of normal rate
   - Increase gradually over 5 minutes
8. **Switch to WebSocket**: For real-time data needs if hitting REST limits

## Error Code Reference

| Error | Code | Type | Description | Recovery |
|-------|------|------|-------------|----------|
| Rate Limit | ERR_RATE_LIMIT | REST | Too many requests | Wait 60 seconds |
| Connection Limit | 429 | WebSocket | Too many connections | Wait 15-60 seconds |
| Subscription Limit | 10305 | WebSocket | Too many channels | Create new connection |
| Unknown Event | 10000 | WebSocket | Invalid event type | Fix event name |
| Unknown Pair | 10001 | WebSocket | Invalid symbol | Fix symbol format |
| Subscription Failed | 10300 | WebSocket | Generic sub error | Check subscription params |
| Already Subscribed | 10301 | WebSocket | Duplicate subscription | Ignore or unsubscribe first |
| Unknown Channel | 10302 | WebSocket | Invalid channel | Fix channel name |
| Unsubscribe Failed | 10400 | WebSocket | Generic unsub error | Check if subscribed |
| Not Subscribed | 10401 | WebSocket | Can't unsub | Ignore |
| Nonce Too Small | 10112 | REST | Nonce not increasing | Fix nonce generation |
| Invalid API Key | 10100 | REST | Wrong API key | Check credentials |
| Invalid Signature | 10111 | REST | Auth signature wrong | Fix signature algorithm |

## Implementation Summary

### Required Client-Side Tracking

Since Bitfinex provides no rate limit headers:

1. **REST API**: Track request timestamps in sliding window
2. **Orders**: Track order submissions in 5-minute window
3. **WebSocket Connections**: Track connection attempts with time windows
4. **WebSocket Subscriptions**: Count subscriptions per connection

### Key Implementation Points

```rust
// Main rate limiters needed
struct BitfinexRateLimits {
    rest_general: RateLimiter,      // 10-90 req/min depending on endpoint
    orders: RateLimiter,             // 1000 orders/5min
    ws_auth_conn: RateLimiter,       // 5 conn/15s
    ws_public_conn: RateLimiter,     // 20 conn/min
    subscriptions: HashMap<ConnId, usize>, // Track per-connection subs
}

const MAX_SUBSCRIPTIONS: usize = 25;  // Conservative limit
const ORDER_WINDOW_SECS: u64 = 300;   // 5 minutes
const REST_WINDOW_SECS: u64 = 60;     // 1 minute
```

## Future-Proofing

Rate limits may change over time:

1. **Don't Hardcode Limits**: Use configurable values
2. **Monitor API Announcements**: Bitfinex may update limits
3. **Implement Adaptive Throttling**: Adjust based on observed limits
4. **Log Rate Limit Errors**: Helps identify when limits change
5. **Test Regularly**: Ensure your implementation still works
6. **Version Your Config**: Track limit changes over time

## Sources

- [Bitfinex Requirements and Limitations](https://docs.bitfinex.com/docs/requirements-and-limitations)
- [Bitfinex REST API General](https://docs.bitfinex.com/docs/rest-general)
- [Bitfinex WebSocket General](https://docs.bitfinex.com/docs/ws-general)
- [Bitfinex WebSocket Public Channels](https://docs.bitfinex.com/docs/ws-public)
- [Bitfinex API Node Issue #242 - Order Rate Limit FAQ](https://github.com/bitfinexcom/bitfinex-api-node/issues/242)
- [Bitfinex API Node Issue #81 - HTTP 429 Error](https://github.com/bitfinexcom/bitfinex-api-node/issues/81)
- [CCXT Issue #6467 - RateLimitExceeded](https://github.com/ccxt/ccxt/issues/6467)
- [Bitfinex API Node Issue #488 - WebSocket Subscribe Limit Error](https://github.com/bitfinexcom/bitfinex-api-node/issues/488)
