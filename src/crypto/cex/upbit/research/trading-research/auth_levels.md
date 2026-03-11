# Upbit Authentication and Permission Levels

Source: https://global-docs.upbit.com/reference/auth
Research date: 2026-03-11

---

## Authentication Method

Upbit uses **JWT (JSON Web Tokens)** for all private/Exchange API authentication.

- **Algorithm:** `HS512` (HMAC with SHA-512)
- **Token type:** `JWT` (Bearer token)
- **Header transmission:** `Authorization: Bearer {jwt_token}`
- There is NO HMAC-SHA256 signing of raw requests as seen in Binance/Bybit style. All auth is JWT-based.

---

## JWT Token Structure

A JWT has three dot-separated Base64-encoded parts: `header.payload.signature`

### JWT Header

```json
{
  "alg": "HS512",
  "typ": "JWT"
}
```

The algorithm **must be HS512**. Other algorithms are not accepted.

### JWT Payload

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `access_key` | string | Always | Your API access key identifier |
| `nonce` | string (UUID) | Always | A unique UUID v4 per request, prevents token reuse |
| `query_hash` | string | When request has parameters | SHA512 hash of the query string or body |
| `query_hash_alg` | string | Optional | Hash algorithm used for `query_hash`. Defaults to `SHA512` |

Example payload for a no-parameter request (e.g., `GET /accounts`):

```json
{
  "access_key": "your_access_key_here",
  "nonce": "550e8400-e29b-41d4-a716-446655440000"
}
```

Example payload for a request with parameters (e.g., `GET /orders/open?market=SGD-BTC`):

```json
{
  "access_key": "your_access_key_here",
  "nonce": "550e8400-e29b-41d4-a716-446655440001",
  "query_hash": "a3f5...(SHA512 of 'market=SGD-BTC')",
  "query_hash_alg": "SHA512"
}
```

### JWT Signature

The signature is computed as:

```
HMAC-SHA512(base64url(header) + "." + base64url(payload), secret_key)
```

**Critical:** The Secret Key is used **as-is** (not Base64-decoded before signing).

---

## Query Hash Construction

The `query_hash` field binds the JWT to specific request parameters, preventing replay attacks with different parameters.

### For GET / DELETE requests (query string parameters)

Hash the raw query string in the exact form it appears in the URL (before URL encoding or reordering):

```
Input:  "market=SGD-BTC&limit=10"
Output: SHA512(input) → hex string → put in query_hash
```

### For POST requests (JSON body)

Convert the JSON body to `key=value&key2=value2` form, then hash:

```
Body JSON: {"market": "SGD-BTC", "side": "bid", "ord_type": "limit"}
Converted: "market=SGD-BTC&side=bid&ord_type=limit"
Output:    SHA512(converted_string) → hex string → put in query_hash
```

### Array parameters

Use bracket notation for arrays:

```
states[]=wait&states[]=watch
```

---

## Request Construction Example

```
1. Build the query string: "market=SGD-BTC"
2. Compute: query_hash = SHA512("market=SGD-BTC")
3. Build JWT payload:
   {
     "access_key": "ACCESS_KEY",
     "nonce": "RANDOM_UUID",
     "query_hash": "computed_sha512_hex",
     "query_hash_alg": "SHA512"
   }
4. Sign JWT with HS512 using SECRET_KEY
5. Send request:
   GET https://sg-api.upbit.com/v1/orders/open?market=SGD-BTC
   Authorization: Bearer <jwt_token>
```

---

## Permission Levels

Upbit API keys have **six distinct permission scopes**, each independently toggleable:

| Permission Name | Allowed Operations |
|----------------|--------------------|
| **View Account** | `GET /accounts` (balances); subscribe to account balance WebSocket stream |
| **Make Orders** | `POST /orders` (create), `POST /orders/test`, `POST /orders/cancel_and_new`, `DELETE /order` (cancel single), `DELETE /orders/uuids` (cancel by IDs), `DELETE /orders/open` (batch cancel) |
| **View Orders** | `GET /order`, `GET /orders/uuids`, `GET /orders/open`, `GET /orders/closed`, `GET /orders/info`; subscribe to order status WebSocket stream |
| **Withdraw** | `POST /withdraws/coin` (withdraw assets), `DELETE /withdraw` (cancel withdrawal) |
| **View Withdrawals** | `GET /withdraw`, `GET /withdraws`, `GET /withdraws/chance`, `GET /withdraws/coin_addresses` |
| **View Deposits** | `GET /deposit`, `GET /deposits`, `GET /deposits/coin_address`, `GET /deposits/coin_addresses`, `POST /deposits/generate_coin_address` |

Permissions are **set at API key creation time** via the Upbit web interface. They cannot be modified after creation via API.

---

## Rate Limits by Authentication Level

### Public (Quotation) API — No authentication required

| Group | Limit | Measurement |
|-------|-------|-------------|
| `market` (trading pair list) | 10 req/sec | Per IP |
| `candle` (OHLCV) | 10 req/sec | Per IP |
| `trade` (recent trades) | 10 req/sec | Per IP |
| `ticker` (current prices) | 10 req/sec | Per IP |
| `orderbook` | 10 req/sec | Per IP |

### Private (Exchange) API — Authentication required

| Group | Limit | APIs Covered | Measurement |
|-------|-------|--------------|-------------|
| `default` | 30 req/sec | All Exchange API endpoints not in specialized groups (accounts, withdrawals, deposits, order queries) | Per account |
| `order` | 8 req/sec | `POST /orders` (create), `POST /orders/cancel_and_new` | Per account |
| `order-test` | 8 req/sec | `POST /orders/test` | Per account |
| `order-cancel-all` | 1 req / 2 seconds | `DELETE /orders/open` (batch cancel) | Per account |

Multiple API keys on the same Upbit account **share** the same rate limit quota — limits apply per account, not per key.

### Rate Limit Response Header

Every API response includes:

```
Remaining-Req: group=default; min=1800; sec=29
```

Fields:
- `group` — the rate limit group for this endpoint
- `min` — remaining requests in the current minute window
- `sec` — remaining requests in the current second window

Both `min` and `sec` limits apply. Whichever is exhausted first triggers throttling.

### Rate Limit Violations

| HTTP Status | Meaning |
|-------------|---------|
| `429` | Too Many Requests — exceeded per-second or per-minute limit |
| `418` | Temporary ban — continued violations after receiving 429 responses |

Repeated 429 violations result in progressively longer temporary IP/account blocks.

### Special Origin Header Restriction

Requests that include an `Origin` HTTP header (typically from browsers or improperly configured clients) are subject to much stricter limits: **1 request per 10 seconds** for both quotation and WebSocket. REST API clients should **not** send the `Origin` header.

---

## WebSocket Rate Limits

| Type | Limit | Measurement |
|------|-------|-------------|
| New connections | 5 connections/sec | Per account (authed) or per IP (public) |
| Data request messages | 5 msg/sec, 100 msg/min | Per account (authed) or per IP (public) |

---

## IP Restrictions

- API keys **can** be restricted to specific IP addresses (IP allowlist).
- IP allowlists are configured at key creation time via the Upbit web interface.
- IP restriction is NOT configurable or queryable via the REST API.
- If a key has IP restrictions and a request comes from an unlisted IP, it will be rejected (4xx error).

---

## Credentials Required

Two values are needed:

| Credential | Description |
|-----------|-------------|
| `access_key` | Public identifier for the API key (goes into JWT payload) |
| `secret_key` | Private secret used to sign the JWT (never transmitted) |

Both are generated together when creating an API key in the Upbit dashboard. There is no concept of separate signing keys or RSA key pairs — HS512 symmetric key signing only.

---

## No Public/Private Key (RSA / Ed25519)

Upbit uses **only** HS512 (symmetric HMAC) signing. RSA and Ed25519 authentication are NOT SUPPORTED.

---

## Sources

- [Authentication Reference](https://global-docs.upbit.com/reference/auth)
- [First Authenticated API Call Guide](https://global-docs.upbit.com/docs/first-exchange-api-call)
- [Rate Limits Reference](https://global-docs.upbit.com/reference/rate-limits)
- [Upbit Global Developer Center](https://global-docs.upbit.com/reference)
