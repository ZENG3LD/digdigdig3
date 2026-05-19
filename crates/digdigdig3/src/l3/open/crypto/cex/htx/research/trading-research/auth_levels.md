# HTX (Huobi) Authentication, Rate Limits & API Key Permissions

## Base URLs

| Purpose       | URL |
|---------------|-----|
| Spot REST      | `https://api.huobi.pro` |
| Spot REST (AWS) | `https://api-aws.huobi.pro` |
| Futures (Coin-M / USDT-M) | `https://api.hbdm.com` |
| Spot WS (market data) | `wss://api.huobi.pro/ws` |
| Spot WS (market MBP) | `wss://api.huobi.pro/feed` |
| Spot WS (account/orders) | `wss://api.huobi.pro/ws/v2` |
| Spot WS (AWS variants) | Same URLs with `api-aws.huobi.pro` |

Use `api-aws.huobi.pro` if your server is on AWS — it has lower latency to HTX infrastructure.

---

## API Key Permissions

Each API key has three independently toggleable permission flags:

| Permission   | Enables |
|--------------|---------|
| **Read**     | All GET/query endpoints — order history, balances, positions, market data |
| **Trade**    | Order placement, order cancellation, transfers, margin borrow/repay |
| **Withdraw** | Withdrawal creation and cancellation |

**Rules:**
- Up to **20 API keys** per user account
- Each key can bind up to **20 IP addresses** (host IPs or CIDR network ranges)
- Keys **without IP binding expire after 90 days** of non-use
- IP binding is strongly recommended for production keys
- Sub-account API keys inherit permission constraints set by the parent account

---

## Authentication Mechanism

All private endpoints use **HmacSHA256 + Base64** signatures.

### Signature Components

| Parameter         | Value / Format |
|-------------------|---------------|
| `AccessKeyId`     | Your API public key |
| `SignatureMethod`  | `HmacSHA256` |
| `SignatureVersion` | `2` |
| `Timestamp`        | UTC timestamp: `YYYY-MM-DDThh:mm:ss` |

Timestamp must be within **5 minutes** of server time. Use `/v1/common/timestamp` to sync.

### 8-Step Signing Process

**Step 1** — Start with HTTP method + newline:
```
GET\n
```

**Step 2** — Add lowercase hostname + newline:
```
api.huobi.pro\n
```

**Step 3** — Add request path + newline:
```
/v1/order/orders\n
```

**Step 4** — URL-encode all query parameters and sort by ASCII order.

Mandatory parameters to always include:
- `AccessKeyId`
- `SignatureMethod`
- `SignatureVersion`
- `Timestamp`

**Step 5** — Concatenate sorted parameters with `&`:
```
AccessKeyId=abc123&SignatureMethod=HmacSHA256&SignatureVersion=2&Timestamp=2024-01-15T10%3A30%3A00&symbol=btcusdt
```

Note: `:` → `%3A`, do NOT encode the `=` within the parameter string — only encode parameter values.

**Step 6** — Assemble the pre-signed string (Steps 1–5 concatenated):
```
GET\n
api.huobi.pro\n
/v1/order/orders\n
AccessKeyId=abc123&SignatureMethod=HmacSHA256&SignatureVersion=2&Timestamp=2024-01-15T10%3A30%3A00&symbol=btcusdt
```

**Step 7** — Compute HMAC-SHA256 over the pre-signed string using your Secret Key, then Base64-encode:
```
4F65x5A2bLyMWVQj3Aqp+B4w+ivaA7n5Oi2SuYtCJ9o=
```

**Step 8** — URL-encode the signature and append to the request URL:
```
https://api.huobi.pro/v1/order/orders?AccessKeyId=abc123&SignatureMethod=HmacSHA256&SignatureVersion=2&Timestamp=2024-01-15T10%3A30%3A00&symbol=btcusdt&Signature=4F65x5A2bLyMWVQj3Aqp%2BB4w%2BivaA7n5Oi2SuYtCJ9o%3D
```

### Critical Rules

1. **GET requests**: Authentication parameters go in the URL query string.
2. **POST requests**: Authentication parameters go in the URL query string; the request body is the JSON payload (never include auth params in the body).
3. Parameter names are case-sensitive.
4. Sort parameters by their byte values in ASCII order (uppercase before lowercase).

### Complete Signed URL Example

```
https://api.huobi.pro/v1/order/orders?AccessKeyId=e2xxxxxx-99xxxxxx-84xxxxxx-7xxxx
  &order-id=1234567890
  &SignatureMethod=HmacSHA256
  &SignatureVersion=2
  &Timestamp=2017-05-11T15%3A19%3A30
  &Signature=4F65x5A2bLyMWVQj3Aqp%2BB4w%2BivaA7n5Oi2SuYtCJ9o%3D
```

---

## Rate Limits

### Spot API

Rate limiting is **UID-based**: all API keys under the same UID share the same rate limit pool per endpoint.

**HTTP Response Headers for Monitoring:**

| Header | Description |
|--------|-------------|
| `X-HB-RateLimit-Requests-Remain` | Remaining requests in current window |
| `X-HB-RateLimit-Requests-Expire` | Milliseconds until current window resets |

**Endpoint-Specific Limits:**

| Endpoint | Method | Limit |
|----------|--------|-------|
| `POST /v1/order/orders/place` | Trade | 50/2s (25 req/sec) |
| `POST /v1/order/batch-orders` | Trade | 25/2s (12.5 req/sec) |
| `POST /v1/order/orders/{id}/submitcancel` | Trade | No rate limit (removed per 2023 announcement) |
| `POST /v1/order/orders/batchCancelOpenOrders` | Trade | ~100/2s |
| `GET /v1/order/openOrders` | Read | 100/2s |
| `GET /v1/order/orders` | Read | 100/2s |
| `GET /v1/order/orders/{id}` | Read | 100/2s |
| `GET /v1/order/matchresults` | Read | 100/2s |
| `POST /v2/algo-orders` | Trade | 20/2s |

**General guidance** (from documentation):
- Private trading endpoints (order placement): ~25 req/sec
- Private query endpoints: ~50 req/sec
- Public market data endpoints: higher limits

### Futures API (Coin-M DM and USDT-M Swap)

| Interface Type | Limit |
|----------------|-------|
| Private REST (per UID) | 72 times/3s (~24 req/sec) |
| Public REST | 60 times/3s (~20 req/sec) |
| Master-Sub account transfers | 10 per minute |

---

## Error Codes Related to Rate Limiting

| Code | Message | Action |
|------|---------|--------|
| `429` | Too Many Requests | Back off, check `X-HB-RateLimit-Requests-Remain` |
| `api-limit-exceeded` | UID rate limit hit | Reduce request frequency |

---

## Testnet / Sandbox

**HTX does not have a public testnet.**

The documentation explicitly states:

> "The testnet has been alive for months, however the active user count is rather low and the cost is high, after considering carefully we decide to shutdown the testnet environment."

**Alternative approach for testing:**
- Use production API with small amounts
- Test market data endpoints (public, no auth required) without risk
- Use paper trading / simulation at the application level

---

## API Key Management

### Creating API Keys

1. Log in to HTX website
2. Navigate to API Management
3. Create new key with desired permissions
4. Optionally bind IP addresses (recommended)
5. Save the Secret Key immediately — it is only shown once

### Key Lifecycle

- Keys without IP binding: expire after **90 days** without use
- Keys with IP binding: do not expire automatically
- Maximum **20 keys** per account

### Sub-Account API Keys

- Sub-accounts can have their own API keys
- Parent account must grant appropriate permissions
- Sub-account keys are subject to the same rate limits at the UID level

---

## v1 vs v2 Response Format

HTX uses two response formats depending on endpoint version:

**v1 endpoints** (most trading endpoints):
```json
{
  "status": "ok",
  "data": { ... },
  "ts": 1630000000000
}
```

**v2 endpoints** (newer endpoints like /v2/algo-orders, /v2/reference/transact-fee-rate):
```json
{
  "code": 200,
  "message": null,
  "data": { ... }
}
```

Error responses:

**v1 error:**
```json
{
  "status": "error",
  "err-code": "invalid-parameter",
  "err-msg": "invalid symbol",
  "ts": 1630000000000
}
```

**v2 error:**
```json
{
  "code": 2002,
  "message": "invalid symbol"
}
```

---

## WebSocket Authentication (Account/Orders Channel)

WebSocket private channel uses a separate authentication flow:

**URL**: `wss://api.huobi.pro/ws/v2`

**Auth message format:**
```json
{
  "action": "req",
  "ch": "auth",
  "params": {
    "authType": "api",
    "accessKey": "your-access-key",
    "signatureMethod": "HmacSHA256",
    "signatureVersion": "2.1",
    "timestamp": "2024-01-15T10:30:00",
    "signature": "base64-encoded-hmac-sha256"
  }
}
```

The WebSocket signature uses the same HmacSHA256 method but with `signatureVersion=2.1` and a different pre-signed string format (using `\n` separators without the HTTP method line).

---

## Sources

- [HTX Spot API Reference — Authentication](https://huobiapi.github.io/docs/spot/v1/en/#authentication)
- [HTX Spot API Reference — Introduction](https://huobiapi.github.io/docs/spot/v1/en/#introduction)
- [HTX Order Rate Limit Adjustment](https://www.htx.com/support/24873931166922)
- [HTX Cancel Order Rate Limit Removal](https://www.htx.com/support/24888369501704)
- [HTX Futures API Access Documentation](https://www.htx.com/support/360000188382)
