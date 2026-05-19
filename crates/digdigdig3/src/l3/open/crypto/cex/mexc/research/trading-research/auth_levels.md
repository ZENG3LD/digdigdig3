# MEXC API — Authentication, Permissions, Rate Limits, Testnet

---

## Base URLs

| API | Base URL |
|-----|----------|
| Spot V3 REST | `https://api.mexc.com` |
| Spot V3 WebSocket | `wss://wbs-api.mexc.com/ws` |
| Futures (Contract V1) REST | `https://contract.mexc.com` |
| Futures WebSocket | Not separately documented |

---

## Authentication Overview

### Endpoint Types

| Type | Auth Required | Usage |
|------|--------------|-------|
| **Public** | No | Market data, exchange info, order book |
| **Signed** | Yes (HMAC SHA256) | Account, trading, transfers |

---

## SPOT API Authentication

### Method: HMAC SHA256

All signed Spot endpoints require:

1. **Header:** `X-MEXC-APIKEY: <your_api_key>`
2. **Query parameter:** `timestamp=<unix_ms>`
3. **Query parameter:** `signature=<hmac_hex>`
4. **Optional:** `recvWindow=<ms>` (default 5000, max 60000)

### Signature Generation

```
signature = HMAC-SHA256(totalParams, secretKey)
```

Where `totalParams` = query string + request body concatenated.

**CRITICAL:** Signature must be **lowercase hex** only. Uppercase will be rejected.

#### Example (GET with query params)

```
GET /api/v3/account?timestamp=1699999999999&recvWindow=5000
totalParams = "timestamp=1699999999999&recvWindow=5000"
signature   = hmac_sha256("timestamp=1699999999999&recvWindow=5000", secret_key)
```

Full URL: `/api/v3/account?timestamp=1699999999999&recvWindow=5000&signature=<hex>`

#### Example (POST with body)

```
POST /api/v3/order
Body: symbol=BTCUSDT&side=BUY&type=LIMIT&quantity=0.001&price=40000&timestamp=1699999999999
totalParams = "symbol=BTCUSDT&side=BUY&type=LIMIT&quantity=0.001&price=40000&timestamp=1699999999999"
signature   = hmac_sha256(totalParams, secret_key)
```

Add `&signature=<hex>` to body.

### Timestamp Validation

Server checks:
```
timestamp < (serverTime + 1000)
AND
(serverTime - timestamp) <= recvWindow
```

Requests outside this window return error `700003`.

---

## FUTURES API Authentication

### Method: HMAC SHA256 (Different from Spot)

Futures uses a different header scheme:

| Header | Value |
|--------|-------|
| `ApiKey` | Your API key |
| `Request-Time` | Unix milliseconds string |
| `Signature` | HMAC SHA256 hex |

**Note:** Header name is `ApiKey` (not `X-MEXC-APIKEY` as in Spot).

### Signature Generation for Futures

```
signature = HMAC-SHA256(accessKey + timestamp + requestParams, secretKey)
```

- For **GET/DELETE**: `requestParams` = parameters sorted alphabetically, joined with `&`
- For **POST**: `requestParams` = raw JSON body string

---

## API Key Permissions

### Spot Permission Scopes

| Permission | Scope | Operations |
|-----------|-------|-----------|
| `SPOT_ACCOUNT_READ` | Account | Read account info, balances, trade history |
| `SPOT_ACCOUNT_WRITE` | Account | Modify account settings |
| `SPOT_DEAL_READ` | Trading | Read orders, open orders |
| `SPOT_DEAL_WRITE` | Trading | Place, cancel orders |
| `SPOT_TRANSFER_READ` | Transfers | Read transfer history |
| `SPOT_TRANSFER_WRITE` | Transfers | Execute transfers between accounts |
| `SPOT_WITHDRAW_READ` | Withdrawals | Read withdrawal history |
| `SPOT_WITHDRAW_WRITE` | Withdrawals | Submit withdrawal requests |

Futures trading has its own permission flags (not separately enumerated in public docs, but required for `/api/v1/private/*` endpoints).

### Key Constraints

| Limit | Value |
|-------|-------|
| Max API keys per account | 30 |
| Max IP whitelist per key | 10 IPs |
| Key expiration (no IP bind) | 90 days |
| Key expiration (with IP bind) | No expiration |
| Max sub-accounts per master | 30 |

---

## Rate Limits

### Spot REST API

**System:** Independent weight buckets per endpoint, enforced on both IP and UID axes.

| Bucket | Limit | Window |
|--------|-------|--------|
| IP-based | 500 weight units | 10 seconds |
| UID-based | 500 weight units | 10 seconds |

**Per-endpoint weights (Spot):**

| Endpoint | IP Weight | UID Weight |
|----------|-----------|-----------|
| `POST /api/v3/order` | 1 | 1 |
| `POST /api/v3/order/test` | 1 | — |
| `DELETE /api/v3/order` | 1 | — |
| `DELETE /api/v3/openOrders` | 1 | — |
| `GET /api/v3/order` | 2 | — |
| `GET /api/v3/openOrders` | 3 | — |
| `GET /api/v3/allOrders` | 10 | — |
| `GET /api/v3/myTrades` | 10 | — |
| `GET /api/v3/account` | 10 | — |
| `POST /api/v3/batchOrders` | 1 | 1 |

**Batch orders additional limit:** `POST /api/v3/batchOrders` — 2 requests/second (separate from weight system).

### Futures REST API

**Rate limits per endpoint:** 20 requests per 2 seconds for most trading/account endpoints.

### WebSocket Limits

| Limit | Value |
|-------|-------|
| Message rate | 100 per second |
| Max streams per connection | 30 |

---

## HTTP Error Responses

### Rate Limit Exceeded

```
HTTP 429 Too Many Requests
Header: Retry-After: <seconds>
```

Repeated violations trigger automated bans:
- First violation: 2-minute ban
- Escalating up to: 3-day ban
- Bans are applied at the **IP level**

### WAF Violation

```
HTTP 403 Forbidden
```

Triggered by suspicious request patterns (automated scanning, etc.).

### Server Error

```
HTTP 5XX
```

Treat as **unknown execution status** — the order may or may not have been placed. Query order status before retrying.

---

## Error Codes

### Authentication Errors

| Code | Message |
|------|---------|
| `602` | Signature verification failed |
| `10001` | User does not exist |
| `10072` | Invalid access key |
| `700003` | Timestamp outside recvWindow |
| `730705` | Maximum 30 API keys per account reached |

### Order Errors

| Code | Message |
|------|---------|
| `30002` | Minimum transaction volume not met |
| `30029` | Maximum open order limit exceeded |

---

## Testnet

**MEXC does NOT have a testnet environment.**

The only validation mechanism is:
- `POST /api/v3/order/test` — Validates Spot order parameters without execution; returns `{}` on success

There is no:
- Testnet base URL
- Paper trading environment
- Simulated order fill responses

---

## Open Order Limits

| Limit | Value |
|-------|-------|
| Max open orders per account (Spot) | 500 |

---

## Key Differences from Binance Auth

| Feature | MEXC Spot | Binance Spot |
|---------|-----------|--------------|
| API key header | `X-MEXC-APIKEY` | `X-MBX-APIKEY` |
| Signature case | **Lowercase only** | Either case accepted |
| Futures header | `ApiKey` (different name) | `X-MBX-APIKEY` (same as Spot) |
| Futures signature input | `accessKey + timestamp + params` | `queryString + body` |
| Testnet | No | Yes (testnet.binance.vision) |
| Key expiration | 90 days without IP bind | No expiration |

---

## Sources

- MEXC Spot V3 API: https://mexcdevelop.github.io/apidocs/spot_v3_en/
- MEXC Contract V1 API: https://mexcdevelop.github.io/apidocs/contract_v1_en/
