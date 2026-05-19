# Upbit API Rate Limits

Comprehensive documentation of Upbit API rate limiting system for implementing robust connector with proper rate limit handling.

---

## Overview

Upbit enforces **per-second request rate limits** to ensure stable service for all users. All rate limits are applied per second, with each API endpoint belonging to a specific rate limit group.

### Core Concepts

- **Rate Limit Groups**: Endpoints grouped by functionality with shared quotas
- **Per-Second Enforcement**: Limits reset every second (not per minute)
- **Separate Tracking**: Quotation APIs tracked by IP, Exchange APIs by account
- **Response Headers**: Remaining quota returned in `Remaining-Req` header
- **Progressive Blocking**: Repeated violations lead to temporary IP/account blocks

---

## 1. Rate Limit Groups

### 1.1 Quotation API (Public Endpoints)

**Measurement**: Per IP address

| Group | Limit | Endpoints |
|-------|-------|-----------|
| **Quotation: market** | 10 req/sec | `/v1/trading-pairs` |
| **Quotation: candle** | 10 req/sec | All `/v1/candles/*` endpoints (seconds, minutes, days, weeks, months, years) |
| **Quotation: trade** | 10 req/sec | `/v1/trades/recent` |
| **Quotation: ticker** | 10 req/sec | `/v1/tickers`, `/v1/tickers/quote` |
| **Quotation: orderbook** | 10 req/sec | `/v1/orderbooks`, `/v1/orderbook-instruments` |

**Key Points**:
- All public market data endpoints limited to 10 requests per second per IP
- Each group (market, candle, trade, ticker, orderbook) has independent quota
- Simultaneous requests to different groups do not affect each other

**Example**:
```
Within 1 second from same IP:
- 10 ticker requests → OK
- 10 candle requests → OK
- 10 orderbook requests → OK
Total: 30 requests OK (different groups)

- 11 ticker requests → 1 request will be rejected (429 error)
```

---

### 1.2 Exchange API (Private Endpoints)

**Measurement**: Per account (shared across all API keys for that account)

| Group | Limit | Endpoints |
|-------|-------|-----------|
| **Exchange: default** | 30 req/sec | `/v1/balances`, `/v1/orders` (GET), `/v1/orders/{order-id}` (GET), `/v1/orders/{order-id}` (DELETE), `/v1/deposits/*`, `/v1/withdrawals/*`, `/v1/orders/info` |
| **Exchange: order** | 8 req/sec | `/v1/orders` (POST - create order), `/v1/orders` (POST - cancel & new) |
| **Exchange: order-test** | 8 req/sec | `/v1/orders/test` (POST) |
| **Exchange: order-cancel-all** | 1 req/2 sec | `/v1/orders` (DELETE - batch cancel) |

**Key Points**:
- Most read operations: 30 requests per second
- Order creation: 8 requests per second
- Batch cancel: 1 request per 2 seconds (very strict)
- Rate limits shared across all API keys of same account

**Example**:
```
Account with 3 API keys:
- Key 1: 15 balance requests
- Key 2: 10 order list requests
- Key 3: 5 order detail requests
Total: 30 requests in 1 second = OK (all from Exchange: default group)

- Key 1: 20 balance requests
- Key 2: 15 order list requests
Total: 35 requests in 1 second = 5 requests rejected (exceeds 30/sec limit)
```

---

### 1.3 WebSocket Connection Limits

| Type | Limit | Description |
|------|-------|-------------|
| **WebSocket: connect** | 5 connections/sec | Connection establishment rate |
| **WebSocket: message** | 5 msg/sec, 100 msg/min | Subscription/unsubscription messages |

**Key Points**:
- Maximum 5 new WebSocket connections per second
- Maximum 5 subscription messages per second
- Maximum 100 subscription messages per minute
- Public and private WebSocket counted separately

---

## 2. Response Headers

### 2.1 Remaining-Req Header

Every API response includes rate limit information:

**Header Name**: `Remaining-Req`

**Format**: `group={group_name}; min={min_value}; sec={sec_value}`

**Example**:
```
Remaining-Req: group=default; min=1800; sec=29
```

### 2.2 Header Fields

| Field | Description | Notes |
|-------|-------------|-------|
| `group` | Rate limit group name | e.g., "default", "order", "candle" |
| `min` | Legacy per-minute value | **DEPRECATED**: Fixed value, ignore this |
| `sec` | Remaining requests this second | **ACTIVE**: Number of requests left in current second |

### 2.3 Interpretation

**Current Second Quota**:
```
Remaining-Req: group=default; min=1800; sec=29
```
- Can make **29 more requests** to "default" group endpoints in current second
- `min=1800` is deprecated and should be ignored

**Quota Exhausted**:
```
Remaining-Req: group=default; min=1800; sec=0
```
- **0 requests remaining** in current second
- Next request will be rejected with 429 error
- Wait for next second to reset

---

## 3. Error Responses

### 3.1 Rate Limit Exceeded (429)

**HTTP Status**: `429 Too Many Requests`

**Response Body**:
```json
{
  "error": {
    "name": "too_many_requests",
    "message": "You have exceeded the request rate limit"
  }
}
```

**Cause**: Exceeded per-second request limit for a rate limit group

**Action**: Wait for next second (quota resets every second)

---

### 3.2 Temporary Block (418)

**HTTP Status**: `418 I'm a teapot`

**Response Body**:
```json
{
  "error": {
    "name": "temporary_block",
    "message": "Your IP/account has been temporarily blocked. Block duration: 300 seconds"
  }
}
```

**Cause**: Continued violations after receiving 429 errors

**Block Duration**: Progressive (increases with repeated violations)
- First block: 60 seconds
- Second block: 300 seconds (5 minutes)
- Third+ block: 600 seconds (10 minutes) or longer

**Action**: Stop sending requests and wait for block duration to expire

**Critical**: If you receive 429 errors, **immediately reduce request rate**. Continuing to send requests will trigger temporary blocks.

---

## 4. Special Restrictions

### 4.1 Origin Header Restriction

**Rule**: Requests including the `Origin` header face stricter limits

**Limit**: **1 request per 10 seconds** for both Quotation and WebSocket APIs

**Affected Requests**:
- Browser-based requests (automatically include `Origin`)
- CORS requests

**Workaround**: For server-side applications, do not include `Origin` header

**Example**:
```python
# Don't include Origin header in server-side requests
headers = {
    "Authorization": f"Bearer {token}",
    "Content-Type": "application/json"
    # Do NOT add: "Origin": "https://example.com"
}
```

---

### 4.2 Travel Rule Verification

**Endpoint**: Travel Rule deposit verification endpoints

**Limit**: **1 request per 10 minutes** per deposit

**Purpose**: Anti-money laundering compliance

**Note**: Only relevant for deposit/withdrawal operations requiring verification

---

## 5. Best Practices

### 5.1 Track Remaining Quota

**Implementation**:
```python
import requests
import time

def make_request(url, headers):
    response = requests.get(url, headers=headers)

    # Parse Remaining-Req header
    remaining_req = response.headers.get('Remaining-Req', '')
    if 'sec=' in remaining_req:
        sec_value = int(remaining_req.split('sec=')[1].split(';')[0])
        print(f"Remaining requests: {sec_value}")

        # Warning if quota low
        if sec_value < 5:
            print(f"WARNING: Only {sec_value} requests left this second")
            time.sleep(1)  # Wait for quota reset

    return response
```

### 5.2 Adaptive Rate Limiting

**Strategy**: Adjust request rate based on remaining quota

```python
class RateLimiter:
    def __init__(self, max_per_second=30):
        self.max_per_second = max_per_second
        self.requests_this_second = 0
        self.current_second = int(time.time())

    def wait_if_needed(self, remaining_quota):
        now = int(time.time())

        # Reset counter if new second
        if now != self.current_second:
            self.requests_this_second = 0
            self.current_second = now

        # Check if should wait
        if remaining_quota < 3 or self.requests_this_second >= self.max_per_second - 1:
            # Wait for next second
            wait_time = 1 - (time.time() - now)
            if wait_time > 0:
                time.sleep(wait_time)
            self.requests_this_second = 0
            self.current_second = int(time.time())

        self.requests_this_second += 1
```

### 5.3 Handle 429 Errors

**Retry Logic**:
```python
import time

def request_with_retry(url, headers, max_retries=3):
    for attempt in range(max_retries):
        response = requests.get(url, headers=headers)

        if response.status_code == 429:
            print(f"Rate limit exceeded (attempt {attempt + 1}/{max_retries})")
            # Wait for next second
            time.sleep(1)
            continue

        if response.status_code == 418:
            print("Temporary block detected")
            # Parse block duration from response
            error_data = response.json()
            message = error_data.get('error', {}).get('message', '')
            # Extract duration (e.g., "Block duration: 300 seconds")
            if 'duration:' in message:
                duration = int(message.split('duration:')[1].split()[0])
                print(f"Waiting {duration} seconds for block to expire")
                time.sleep(duration + 1)
            else:
                time.sleep(60)  # Default wait
            continue

        return response

    raise Exception("Max retries exceeded")
```

### 5.4 Distribute Requests Across Groups

**Strategy**: Use different endpoints to utilize separate quotas

```python
# Instead of:
# - 50 calls to /v1/tickers in 1 second (exceeds 10/sec)

# Do:
# - 10 calls to /v1/tickers (ticker group)
# - 10 calls to /v1/orderbooks (orderbook group)
# - 10 calls to /v1/trades/recent (trade group)
# - 10 calls to /v1/candles/minutes/1 (candle group)
# Total: 40 requests OK (different groups)
```

### 5.5 Use WebSocket for Real-Time Data

**Recommendation**: Use WebSocket for high-frequency market data updates

**Benefits**:
- No per-request rate limits (only connection and subscription limits)
- Real-time updates without polling
- Reduced server load

**Example**:
```python
# Instead of polling every second:
while True:
    ticker = requests.get("https://sg-api.upbit.com/v1/tickers?markets=SGD-BTC")
    time.sleep(1)  # 10 req/sec limit

# Use WebSocket:
import websocket
ws = websocket.WebSocketApp("wss://sg-api.upbit.com/websocket/v1")
ws.send('[{"ticket":"test"},{"type":"ticker","codes":["SGD-BTC"]}]')
# Receive real-time updates without rate limits
```

---

## 6. Rate Limit by Endpoint

### 6.1 Quotation API Summary

| Endpoint | Group | Limit | Auth |
|----------|-------|-------|------|
| `/v1/trading-pairs` | market | 10/sec | No |
| `/v1/candles/seconds` | candle | 10/sec | No |
| `/v1/candles/minutes/*` | candle | 10/sec | No |
| `/v1/candles/days` | candle | 10/sec | No |
| `/v1/candles/weeks` | candle | 10/sec | No |
| `/v1/candles/months` | candle | 10/sec | No |
| `/v1/candles/years` | candle | 10/sec | No |
| `/v1/trades/recent` | trade | 10/sec | No |
| `/v1/tickers` | ticker | 10/sec | No |
| `/v1/tickers/quote` | ticker | 10/sec | No |
| `/v1/orderbooks` | orderbook | 10/sec | No |
| `/v1/orderbook-instruments` | orderbook | 10/sec | No |

### 6.2 Exchange API Summary

| Endpoint | Method | Group | Limit | Auth |
|----------|--------|-------|-------|------|
| `/v1/balances` | GET | default | 30/sec | Yes |
| `/v1/orders/info` | GET | default | 30/sec | Yes |
| `/v1/orders` | GET | default | 30/sec | Yes |
| `/v1/orders/{order-id}` | GET | default | 30/sec | Yes |
| `/v1/orders` | POST | order | 8/sec | Yes |
| `/v1/orders/test` | POST | order-test | 8/sec | Yes |
| `/v1/orders/{order-id}` | DELETE | default | 30/sec | Yes |
| `/v1/orders` | DELETE | order-cancel-all | 1/2sec | Yes |
| `/v1/deposits/*` | All | default | 30/sec | Yes |
| `/v1/withdrawals/*` | All | default | 30/sec | Yes |

---

## 7. Comparison with Other Exchanges

| Exchange | Rate Limit System | Measurement | Reset Period |
|----------|------------------|-------------|--------------|
| **Upbit** | Per-group, per-second | IP (public) / Account (private) | 1 second |
| **Binance** | Weight-based, per-minute | IP (public) / UID (private) | 1 minute |
| **KuCoin** | Weight-based, 30-second window | IP (public) / UID (private) | 30 seconds |
| **Coinbase** | Tier-based, per-second | User tier | 1 second |

**Upbit Characteristics**:
- **Strictest**: 1-second reset (vs. 30s-1min for others)
- **Simpler**: Group-based (vs. weight calculation)
- **Lower Limits**: 10 req/sec public (vs. 1200/min for Binance)
- **Progressive Blocking**: Aggressive enforcement with temporary blocks

---

## 8. Implementation Checklist

### 8.1 Essential Features

- [ ] Parse `Remaining-Req` header from every response
- [ ] Extract `sec` value (ignore deprecated `min` value)
- [ ] Track remaining quota per rate limit group
- [ ] Implement per-second request throttling
- [ ] Handle 429 errors with 1-second retry delay
- [ ] Handle 418 errors with progressive backoff
- [ ] Log rate limit violations for monitoring

### 8.2 Advanced Features

- [ ] Adaptive rate limiting based on remaining quota
- [ ] Separate quota tracking per rate limit group
- [ ] Request queuing with priority
- [ ] WebSocket fallback for high-frequency data
- [ ] Monitoring dashboard for quota utilization
- [ ] Alert when approaching rate limits

### 8.3 Testing Considerations

- [ ] Test behavior when quota exhausted (429 response)
- [ ] Verify retry logic for 429 errors
- [ ] Test temporary block handling (418 response)
- [ ] Confirm per-second quota reset
- [ ] Validate group independence (candle vs ticker)
- [ ] Test rate limiting with multiple API keys

---

## 9. Summary Table

### Quick Reference: Rate Limits by Group

| Group | Type | Measurement | Limit | Endpoints |
|-------|------|-------------|-------|-----------|
| **market** | Quotation | Per IP | 10/sec | Trading pairs list |
| **candle** | Quotation | Per IP | 10/sec | All candlestick data |
| **trade** | Quotation | Per IP | 10/sec | Recent trades |
| **ticker** | Quotation | Per IP | 10/sec | Ticker data |
| **orderbook** | Quotation | Per IP | 10/sec | Orderbook data |
| **default** | Exchange | Per Account | 30/sec | Balances, orders (read), deposits, withdrawals |
| **order** | Exchange | Per Account | 8/sec | Create order |
| **order-test** | Exchange | Per Account | 8/sec | Test order creation |
| **order-cancel-all** | Exchange | Per Account | 1/2sec | Batch cancel orders |

### Error Codes Quick Reference

| Code | Status | Meaning | Action |
|------|--------|---------|--------|
| 429 | Too Many Requests | Rate limit exceeded | Wait 1 second, retry |
| 418 | I'm a teapot | Temporary block | Wait for block duration |

### Header Format

```
Remaining-Req: group={group}; min={deprecated}; sec={remaining}
```

Example: `Remaining-Req: group=default; min=1800; sec=25`

---

## Sources

- [Upbit Open API - Rate Limits](https://global-docs.upbit.com/reference/rate-limits)
- [Upbit Open API - REST API Guide](https://global-docs.upbit.com/reference/rest-api-guide)
- [GitHub - CCXT Upbit Rate Limit Issue](https://github.com/ccxt/ccxt/issues/4604)
- [Tardis.dev - Upbit Data Documentation](https://docs.tardis.dev/historical-data-details/upbit)

---

**Document Version**: 1.0
**Last Updated**: 2026-01-20
**Research Completed By**: Claude Code Research Agent
