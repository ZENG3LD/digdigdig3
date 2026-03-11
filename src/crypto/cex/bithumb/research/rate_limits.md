# Bithumb API Rate Limiting Documentation

**Last Updated**: 2026-01-20
**API Version**: V1.0.0
**Official Docs**: https://github.com/bithumb-pro/bithumb.pro-official-api-docs

## Overview

Bithumb Pro implements basic rate limiting with minimal documentation. The exchange is known for infrastructure issues including frequent 504 gateway timeouts that are separate from rate limiting.

---

## 1. REST API Limits

### 1.1 General Limits

**Official Documentation Quote:**
> "Api request rate limit: 10 time/s for create/cancel order"

- **Order Operations**: 10 requests/second
  - Applies to: Create order, Cancel order
  - Type: Per API key (likely)
  - Enforcement: Server-side

**CCXT Implementation:**
- **Default Rate Limit**: 500ms between requests
- **Translates to**: ~2 requests/second (conservative)
- **Type**: Client-side throttling

### 1.2 Endpoint-Specific Limits

| Endpoint Category | Documented Limit | Notes |
|------------------|------------------|-------|
| Create/Cancel Order | 10 req/s | Explicitly documented |
| Market Data | Not specified | Use conservative approach |
| Account Endpoints | Not specified | Use conservative approach |
| Other Private Endpoints | Not specified | Use conservative approach |

**Important**: Only order creation/cancellation has documented limits. All other endpoints lack explicit rate limit specifications.

### 1.3 IP-based vs API-key-based Limits

**Status**: Not explicitly documented

Based on standard exchange practices and the documentation context:
- Order operations: Likely **API-key-based** (10 req/s)
- Public endpoints: Unknown (possibly IP-based)

**Recommendation**: Assume API-key-based for all private endpoints, IP-based for public endpoints until confirmed otherwise.

---

## 2. Response Headers

### 2.1 Rate Limit Headers

According to CCXT library research and industry patterns, Bithumb may return:

```
X-RateLimit-Remaining: <number>
X-RateLimit-Requested-Tokens: <number>
X-RateLimit-Burst-Capacity: 140
X-RateLimit-Replenish-Rate: 140
```

**Verification Status**: Headers mentioned in CCXT codebase but NOT confirmed in official Bithumb documentation.

**Implementation Note**: Do not rely on these headers for rate limit management. Use client-side throttling instead.

### 2.2 Standard Response Format

All Bithumb API responses follow this structure:

```json
{
  "code": "0",
  "msg": "success",
  "success": true,
  "data": { ... },
  "params": {}
}
```

Rate limit errors would likely use the same format with a different `code` value.

---

## 3. Error Handling

### 3.1 HTTP Status Codes

**Rate Limit Status Code**: Likely **429 Too Many Requests** (standard HTTP)

**Status**: Not explicitly documented by Bithumb

Based on industry standards:
- **429**: Rate limit exceeded
- **504**: Gateway timeout (infrastructure issue, NOT rate limiting)

### 3.2 Rate Limit Error Response Format

**Expected format** (not officially documented):

```json
{
  "code": "<error_code>",
  "msg": "rate limit exceeded",
  "success": false,
  "data": null,
  "params": {}
}
```

**Known Error Codes** (from GitHub issues, not rate limit specific):

| Code | Meaning |
|------|---------|
| 0 | Success |
| -99 | Invalid ApiKey |
| 9002 | Verify signature failed |
| 9006 | No server |
| 10002 | Invalid apikey |
| 20038 | USDT withdrawal error |
| 200006 | Unable to find User Account Data (WebSocket) |

**Rate limit specific error code**: Unknown - not documented

### 3.3 Retry Handling

**Recommended approach:**

1. **Detect rate limit**:
   - HTTP 429 status (if returned)
   - Specific error code in response body (unknown)

2. **Retry strategy**:
   - Exponential backoff starting at 1 second
   - Max retry delay: 32 seconds
   - Max retries: 5

3. **Fallback**:
   - Client-side rate limiting with 500ms delay between requests
   - Reduce to 10 req/s for order operations

---

## 4. WebSocket Limits

### 4.1 Connection Limits

**Documented Limit**:
> "one account just have one authentication's connection with private topic in the same moment"

- **Private Topics**: 1 authenticated connection per account
- **Public Topics**: Not specified

### 4.2 Subscription Limits

**Status**: Not documented

- Maximum subscriptions per connection: Unknown
- Message rate limits: Unknown

### 4.3 WebSocket Heartbeat

**Required**: Yes

**Mechanism**:
```json
// Client sends
{"cmd": "ping"}

// Server responds
{"code": "0", "msg": "pong"}
```

**Purpose**: Keep connection alive
**Interval**: Not specified (recommend 30-60 seconds)

### 4.4 WebSocket Endpoints

- **Base URL**: `wss://global-api.bithumb.pro/message/realtime`
- **Connection limit**: 1 authenticated connection for private topics per account

---

## 5. Infrastructure Issues (Separate from Rate Limits)

### 5.1 Known 504 Gateway Timeout Problem

**GitHub Issue**: [#114 - Getting 504: Gateway time-out errors while REST request](https://github.com/bithumb-pro/bithumb.pro-official-api-docs/issues/114)

**Problem Details**:
- **Reported**: June 21, 2023
- **Frequency**: ~20% of requests (2 out of 10)
- **Affects**: Both public and private endpoints
- **Status**: Unresolved (no official response)

**Example**:
```
GET https://global-openapi.bithumb.pro/openapi/v1/spot/orderBook?symbol=XELS-USDT
Response: 504 Gateway Timeout (Cloudflare error page)
```

### 5.2 504 vs Rate Limiting

**Critical Distinction**:

| Issue | HTTP Code | Cause | Solution |
|-------|-----------|-------|----------|
| Rate Limit | 429 | Too many requests | Wait, then retry |
| Infrastructure | 504 | Server/gateway timeout | Retry with backoff, consider circuit breaker |

**504 Errors are NOT rate limits** - they indicate:
- Backend server overload
- Network issues between gateway and origin server
- Slow query processing
- Infrastructure capacity problems

### 5.3 Handling 504 Errors

**Recommended approach**:

1. **Separate error handling**:
   - 429 → Rate limit backoff
   - 504 → Infrastructure retry with longer delays

2. **Circuit breaker pattern**:
   - Track 504 error rate
   - If >15% of requests fail with 504: temporarily pause API calls
   - Gradually resume after cooldown period

3. **Monitoring**:
   - Log 504 frequency separately from rate limits
   - Alert if 504 rate exceeds threshold
   - Consider switching to WebSocket for critical data

---

## 6. Implementation Notes

### 6.1 Recommended Rate Limiting Strategy

Given the sparse documentation and known infrastructure issues:

**1. Client-Side Rate Limiting (Primary)**

```rust
// Conservative approach
const DEFAULT_DELAY_MS: u64 = 500;  // 2 req/s
const ORDER_DELAY_MS: u64 = 100;    // 10 req/s for orders

// Per-endpoint configuration
match endpoint {
    Endpoint::CreateOrder | Endpoint::CancelOrder => {
        // 10 req/s limit
        delay(ORDER_DELAY_MS).await;
    }
    _ => {
        // Conservative 2 req/s for everything else
        delay(DEFAULT_DELAY_MS).await;
    }
}
```

**2. Adaptive Rate Limiting (Secondary)**

Monitor response headers (if present) and adjust delays dynamically:

```rust
if let Some(remaining) = headers.get("X-RateLimit-Remaining") {
    if remaining.parse::<u32>()? < 5 {
        // Increase delay when approaching limit
        delay(1000).await;
    }
}
```

**3. Error-Based Backoff (Tertiary)**

```rust
match response.status() {
    429 => {
        // Rate limit hit
        exponential_backoff(attempt).await;
    }
    504 => {
        // Infrastructure issue - longer delay
        delay(5000).await;
        check_circuit_breaker();
    }
    _ => {}
}
```

### 6.2 Testing Considerations

**Challenge**: Cannot reliably test rate limits due to:
- Incomplete documentation
- Unknown exact thresholds for most endpoints
- High rate of 504 errors complicating testing

**Approach**:
1. Start with conservative limits (500ms delay)
2. Monitor production for 429 errors
3. Gradually reduce delays if no rate limit errors occur
4. Always respect the 10 req/s limit for order operations

### 6.3 Production Recommendations

1. **Use CCXT's 500ms default** as baseline
2. **Implement circuit breaker** for 504 errors
3. **Log all rate limit responses** to gather empirical data
4. **Monitor 504 vs 429 separately** in metrics
5. **Consider WebSocket** for high-frequency market data
6. **Don't rely on response headers** - use client-side throttling

### 6.4 Comparison with Other Exchanges

| Exchange | Order Limit | General Limit | Documentation Quality |
|----------|-------------|---------------|----------------------|
| Binance | 50 req/s | 1200 req/min | Excellent |
| KuCoin | 3000 req/30s | Varies by endpoint | Good |
| Bybit | 10 req/s | Varies by tier | Excellent |
| **Bithumb** | **10 req/s** | **Unknown** | **Poor** |

Bithumb's rate limiting documentation is significantly less comprehensive than competitors.

---

## 7. Summary

### What We Know

1. Order operations: 10 req/s limit (documented)
2. WebSocket: 1 authenticated connection per account (documented)
3. Heartbeat: ping/pong required (documented)
4. Infrastructure issues: Frequent 504 errors (confirmed)

### What We Don't Know

1. Rate limits for non-order endpoints
2. IP-based vs API-key-based distinction
3. Exact error codes for rate limiting
4. Response headers (if any)
5. Burst allowances
6. Rate limit reset windows

### Recommended Implementation

```rust
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub struct BithumbRateLimiter {
    last_request: Instant,
    delay_ms: u64,
}

impl BithumbRateLimiter {
    pub fn new() -> Self {
        Self {
            last_request: Instant::now(),
            delay_ms: 500, // Conservative default
        }
    }

    pub async fn wait_for_order_endpoint(&mut self) {
        // 10 req/s for orders
        self.wait_with_delay(100).await;
    }

    pub async fn wait_for_general_endpoint(&mut self) {
        // Conservative 2 req/s for everything else
        self.wait_with_delay(500).await;
    }

    async fn wait_with_delay(&mut self, delay_ms: u64) {
        let elapsed = self.last_request.elapsed().as_millis() as u64;
        if elapsed < delay_ms {
            sleep(Duration::from_millis(delay_ms - elapsed)).await;
        }
        self.last_request = Instant::now();
    }
}
```

### Key Takeaways

1. **Be Conservative**: Due to sparse documentation, use cautious rate limits
2. **Separate 504 from 429**: Infrastructure issues are distinct from rate limiting
3. **Client-Side Throttling**: Don't rely on server-provided headers
4. **Monitor and Adapt**: Log all errors and adjust based on production data
5. **Consider Alternatives**: WebSocket for market data, REST for trading only

---

## Sources

- [Bithumb Pro Official API Documentation](https://github.com/bithumb-pro/bithumb.pro-official-api-docs)
- [REST API Documentation](https://github.com/bithumb-pro/bithumb.pro-official-api-docs/blob/master/rest-api.md)
- [WebSocket API Documentation](https://github.com/bithumb-pro/bithumb.pro-official-api-docs/blob/master/ws-api.md)
- [GitHub Issue #114 - 504 Gateway Timeout Errors](https://github.com/bithumb-pro/bithumb.pro-official-api-docs/issues/114)
- [CCXT Bithumb Implementation](https://github.com/ccxt/ccxt/blob/master/python/ccxt/async_support/bithumb.py)
- [CCXT Manual - Rate Limiting](https://github.com/ccxt/ccxt/wiki/manual)
- [HTTP 429 Too Many Requests - MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Status/429)
- [HTTP 504 Gateway Timeout - MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Status/504)

---

## Changelog

- **2026-01-20**: Initial research compiled from official docs, CCXT implementation, and GitHub issues
