# Gate.io APIv4 — Authentication, Permissions, Rate Limits

---

## 1. Base URLs

| Environment | URL |
|-------------|-----|
| Live (all) | `https://api.gateio.ws/api/v4` |
| Live (futures alt) | `https://fx-api.gateio.ws/api/v4` |
| Testnet (futures only) | `https://fx-api-testnet.gateio.ws/api/v4` |
| Testnet (all markets) | `https://api-testnet.gateapi.io/api/v4` |

> **Spot Testnet:** Available via `https://api-testnet.gateapi.io/api/v4`.
> **Futures Testnet:** `https://fx-api-testnet.gateio.ws/api/v4` — dedicated testnet environment.
> **Deprecated:** `*.gateio.io` base URLs are deprecated; use the URLs above.

---

## 2. Authentication Method

Gate.io APIv4 uses **HMAC-SHA512** signed requests.

### 2.1 Required Headers

Every authenticated request must include exactly these three headers:

| Header | Value |
|--------|-------|
| `KEY` | Your API key string |
| `Timestamp` | Current Unix time in **seconds** (integer string) |
| `SIGN` | HMAC-SHA512 signature (hex-encoded) |

> Timestamp must be within **60 seconds** of server time. Requests outside this window are rejected.

---

### 2.2 Signature Generation

The signature string is assembled by concatenating:

```
{HTTP_METHOD}\n
{REQUEST_PATH}\n
{QUERY_STRING}\n
{HexEncode(SHA512(REQUEST_BODY))}\n
{TIMESTAMP}
```

**Fields:**
- `HTTP_METHOD`: uppercase, e.g. `GET`, `POST`, `DELETE`, `PATCH`
- `REQUEST_PATH`: path only, no host, e.g. `/api/v4/spot/orders`
- `QUERY_STRING`: URL-encoded query string (empty string `""` if none)
- `HexEncode(SHA512(REQUEST_BODY))`: SHA-512 hash of the raw request body bytes, hex-encoded. For GET requests with no body, hash of empty string: `cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e`
- `TIMESTAMP`: same value as the `Timestamp` header

**Signature computation:**
```
SIGN = HexEncode(HMAC_SHA512(api_secret, signature_string))
```

### 2.3 Example (Python pseudocode)

```python
import hashlib, hmac, time

api_key = "your_api_key"
api_secret = "your_api_secret"

method = "POST"
path = "/api/v4/spot/orders"
query_string = ""
body = '{"currency_pair":"BTC_USDT","side":"buy","amount":"0.001","price":"40000","type":"limit"}'
timestamp = str(int(time.time()))

body_hash = hashlib.sha512(body.encode()).hexdigest()
signature_string = f"{method}\n{path}\n{query_string}\n{body_hash}\n{timestamp}"
sign = hmac.new(api_secret.encode(), signature_string.encode(), hashlib.sha512).hexdigest()

headers = {
    "KEY": api_key,
    "Timestamp": timestamp,
    "SIGN": sign,
    "Content-Type": "application/json"
}
```

---

## 3. API Key Permissions

Each API key can have independent permissions configured at creation. Each permission group can be set to:
- **Disabled** (no access)
- **Read-only** (GET requests only)
- **Read-write** (all HTTP methods)

### Permission Groups

| Permission | Scope | Required For |
|------------|-------|--------------|
| Spot | Spot trading account | `POST/DELETE/PATCH /spot/orders`, `GET /spot/accounts` |
| Margin | Margin trading | `POST /margin/loans`, `GET /margin/accounts` |
| Futures | Perpetual and delivery futures | `/futures/{settle}/orders`, `/futures/{settle}/positions` |
| Options | Options trading | `/options/orders` |
| Wallet | Wallet operations, transfers | `POST /wallet/transfers`, `GET /wallet/deposits` |
| Withdrawal | Fund withdrawals | `POST /withdrawals` |
| Sub-account | Sub-account management | `/sub_accounts/**`, `/wallet/sub_account_transfers` |

> **NOTE:** All GET (read) operations require the relevant permission set to at least read-only. All POST/DELETE/PATCH (write) operations require read-write for the relevant permission group.

> API keys are **not** separated by trading type. One key can hold multiple permission groups simultaneously.

### Per-Key Limits

- Maximum **20 API keys** per account
- Each key can have an **IP whitelist** (max 20 IPv4 addresses)
- If IP whitelist is set, requests from non-whitelisted IPs are rejected

---

## 4. Rate Limits

Rate limits changed effective **January 22, 2024**. The new system uses per-endpoint limits based on UID (for private endpoints) or IP (for public endpoints).

### 4.1 Spot Trading Rate Limits

| Endpoint Type | Limit | Basis |
|---------------|-------|-------|
| Public endpoints | 200 requests / 10 seconds (per endpoint) | IP address |
| Private endpoints (general) | 200 requests / 10 seconds (per endpoint) | UID |
| `POST /spot/orders` (place order) | 10 requests / second | UID + Market pair |
| `PATCH /spot/orders/{id}` (amend) | 10 requests / second | UID + Market pair |
| `DELETE /spot/orders/{id}` (cancel single) | 200 requests / second | UID |
| `DELETE /spot/orders` (cancel all) | 200 requests / 10 seconds | UID |
| `POST /spot/batch_orders` | 10 requests / second | UID |
| `POST /spot/cancel_batch_orders` | 200 requests / second | UID |
| `POST /spot/amend_batch_orders` | 10 requests / second | UID |

#### Low Fill Ratio Rate Limiting (Spot)

Gate.io monitors hourly fill ratios. If a strategy has a low fill ratio (frequent order placement/cancellation without fills), `POST /spot/orders` and `PATCH /spot/orders/{id}` are temporarily capped at:

> **10 requests / 10 seconds** (per UID)

This cap is automatically lifted within 1 hour once trading behavior improves.

---

### 4.2 Perpetual Futures Rate Limits

| Endpoint Type | Limit | Basis |
|---------------|-------|-------|
| Public endpoints | 200 requests / 10 seconds (per endpoint) | IP address |
| Private endpoints (general) | 200 requests / 10 seconds (per endpoint) | UID |
| `POST /futures/{settle}/orders` (place) | 100 requests / second | UID |
| `PATCH /futures/{settle}/orders/{id}` (amend) | 100 requests / second | UID |
| `DELETE /futures/{settle}/orders/{id}` (cancel) | 200 requests / second | UID |
| `DELETE /futures/{settle}/orders` (cancel all) | 200 requests / 10 seconds | UID |
| `POST /futures/{settle}/batch_orders` | 100 requests / second | UID |

#### Low Fill Ratio Rate Limiting (Futures)

Similar policy as spot: if low fill ratio detected, `POST /futures/{settle}/orders` and `PATCH /futures/{settle}/orders/{id}` are capped at:

> **10 requests / 10 seconds** (per UID)

Automatically lifted within **2 hours** once behavior meets efficiency benchmarks.

Effective: August 5, 2024 (8:00 UTC).

---

### 4.3 Wallet Rate Limits

| Endpoint | Limit | Basis |
|----------|-------|-------|
| `POST /withdrawals` | 10 requests / 10 seconds | UID |
| `POST /withdrawals/push` | 1 request / 10 seconds | UID |
| `POST /wallet/sub_account_transfers` | 80 requests / 10 seconds | UID |
| `GET /wallet/sub_account_balances` | 80 requests / 10 seconds | UID |
| Other wallet endpoints | 200 requests / 10 seconds | UID |

---

### 4.4 Rate Limit Headers

Gate.io returns rate limit information in response headers (X-Gate-RateLimit headers). Check these to avoid 429 errors.

---

## 5. Testnet Details

### 5.1 Futures Testnet

- **URL:** `https://fx-api-testnet.gateio.ws/api/v4`
- **Supports:** USDT-settled and BTC-settled perpetual futures
- **Funding:** Test account funded with test tokens (not real)
- **Auth:** Uses separate testnet API keys (created on testnet portal)
- **Endpoints:** Same paths as production: `/futures/{settle}/orders`, etc.

### 5.2 Spot Testnet

- **URL:** `https://api-testnet.gateapi.io/api/v4`
- **Supports:** Spot and margin trading simulation
- **Availability:** Limited — not all pairs may be available on testnet

> **NOT SUPPORTED:** Options and delivery futures on testnet may have limited or no support. Verify endpoint availability before relying on testnet for these products.

---

## 6. Error Codes

Common API error response format:

```json
{
  "label": "INVALID_PARAM_VALUE",
  "message": "Invalid currency_pair"
}
```

| HTTP Status | Meaning |
|-------------|---------|
| 200 | Success |
| 400 | Bad request (invalid params) |
| 401 | Authentication failure (bad KEY, SIGN, or Timestamp) |
| 403 | Permission denied (key lacks required permission) |
| 404 | Resource not found |
| 429 | Rate limit exceeded |
| 500 | Internal server error |

Common error labels:

| Label | Description |
|-------|-------------|
| `INVALID_PARAM_VALUE` | Invalid parameter value |
| `MISSING_REQUIRED_PARAM` | Required parameter missing |
| `INVALID_CREDENTIALS` | API key/signature invalid |
| `FORBIDDEN` | Permission not granted for this operation |
| `TOO_MANY_REQUESTS` | Rate limit exceeded |
| `ORDER_NOT_FOUND` | Order ID does not exist |
| `BALANCE_NOT_ENOUGH` | Insufficient balance |
| `MARKET_CLOSE` | Market is closed |

---

## 7. What Is NOT Supported

| Feature | Status |
|---------|--------|
| WebSocket order placement (spot) | **NOT supported** — REST only for spot orders |
| Spot testnet (full) | **Partial** — limited availability |
| Order book subscriptions via REST | Not applicable — use WebSocket |
| OCO orders (One-Cancels-Other) | **NOT available** — use price_orders for TP/SL separately |
| Trailing stop orders | **NOT available** natively in REST API |
| Conditional orders chaining | Not supported |
| Options testnet | Not confirmed available |

> WebSocket order placement IS supported for futures via the Futures WebSocket API (`wss://fx-ws.gateio.ws/v4/ws/usdt`), but spot order placement via WebSocket is not available.

---

## 8. Additional Notes

### 8.1 Content-Type

All POST/PATCH requests with a JSON body must include:
```
Content-Type: application/json
```

### 8.2 Idempotency

Use the `text` field (prefixed with `"t-"`) as a client-side order ID for deduplication. Gate.io does not have a formal idempotency key mechanism for REST order creation.

For sub-account transfers, `client_order_id` provides idempotency.

### 8.3 Time Synchronization

The `Timestamp` header must be within **60 seconds** of Gate.io server time. Use NTP or sync with `GET /spot/time` (public endpoint) to get server time.

### 8.4 STP (Self-Trade Prevention)

1. Create an STP group: `POST /account/stp_groups`
2. Add users to the group: `POST /account/stp_groups/{stp_id}/users`
3. Use `stp_id` + `stp_act` in order requests

---

## Sources

- [Gate API v4 Official Docs](https://www.gate.com/docs/developers/apiv4/en/)
- [Gate REST v4 GitHub](https://github.com/gateio/rest-v4)
- [Gate.io Rate Limit Announcement Jan 2024](https://www.gate.com/announcements/article/33995)
- [Gate.io Perpetual Futures Rate Limit Aug 2024](https://www.coincarp.com/exchange/announcement/gate-io-38255/)
- [uTrading Gate.io API Key Guide](https://help.utrading.io/en/exchange/create-api/gate.io-api-import-guide)
- [TabTrader Gate.io API Key Guide](https://tabtrader.com/helpcenter/web/api-keys/gateio)
