# Gemini Exchange — Authentication and Permission Levels

Source: https://docs.gemini.com/authentication/api-key, https://docs.gemini.com/roles, https://docs.gemini.com/rate-limit
Retrieved: 2026-03-11

---

## Authentication Method

### Algorithm

**HMAC-SHA384** — All private API requests are signed using HMAC with SHA-384 digest.

This is NOT RSA, Ed25519, or HMAC-SHA256. It is specifically **HMAC-SHA384**.

---

### How Requests Are Signed

Private API requests on Gemini use an unconventional approach: **the request body is empty**. All parameters are transmitted in HTTP headers, NOT in the POST body.

#### Step-by-Step Signing Procedure

1. **Build the JSON payload** containing at minimum:
   ```json
   {
     "request": "/v1/order/new",
     "nonce": 1234567890123,
     ... (additional endpoint parameters)
   }
   ```

2. **Base64-encode** the raw JSON string:
   ```
   payload_b64 = base64_encode(json_string)
   ```
   Note: No URL-safe encoding, standard base64. Hashing is performed directly on the base64 string with no normalization — exactly what is sent as the header is what the server verifies.

3. **Compute HMAC-SHA384** signature:
   ```
   signature = hex( HMAC_SHA384(key=api_secret, message=payload_b64) )
   ```
   The output must be **hex-encoded** (lowercase hex string).

4. **Send the request** with an empty body and these headers:

| Header | Value |
|--------|-------|
| `Content-Type` | `text/plain` |
| `Content-Length` | `0` |
| `Cache-Control` | `no-cache` |
| `X-GEMINI-APIKEY` | Your API key (the public identifier) |
| `X-GEMINI-PAYLOAD` | Base64-encoded JSON payload |
| `X-GEMINI-SIGNATURE` | Hex-encoded HMAC-SHA384 of the base64 payload |

#### Important Notes
- The POST body is always **empty** — parameters are in headers only.
- `Content-Type: text/plain` and `Content-Length: 0` are required.
- The `request` field in the payload must exactly match the endpoint path being called (e.g. `"/v1/order/new"`).
- Hashes are taken on the base64 string directly. Do NOT hash the raw JSON.

---

### Nonce

The `nonce` field is a **required field in the JSON payload** for all private endpoints.

#### Requirements
- Must be an integer that never repeats.
- Must **strictly increase** with each request on a given session.
- Two valid approaches:
  1. **Millisecond timestamp** (recommended) — use current Unix time in milliseconds
  2. **Sequential counter** — any increasing integer works

#### Time-Based Nonce Constraint
- If using a time-based nonce, it must be within **±30 seconds** of the server's Unix epoch timestamp.

#### Per-Session Semantics
- The nonce is tracked **per API key session**, not globally.
- This allows independent multithreaded applications to use separate API keys without nonce collision.

---

### Payload `request` Field

The payload must include a `request` field whose value is the exact API endpoint path:

```json
{
  "request": "/v1/order/new",
  "nonce": 1741648800000,
  "symbol": "btcusd",
  "amount": "0.01",
  "price": "50000",
  "side": "buy",
  "type": "exchange limit"
}
```

---

### Code Example Languages

Official code examples are provided in the documentation for:
- cURL
- Python
- JavaScript
- C#
- Kotlin
- Objective-C
- PHP
- Ruby
- Swift
- Go
- Java
- **Rust**
- Scala

---

## API Key Types

| Key Type | Prefix | Scope |
|----------|--------|-------|
| Account-level key | `account-` | Operates on one specific account |
| Master key | `master-` | Can operate on any sub-account by specifying `account` parameter |

Master keys can call any endpoint and act on behalf of any sub-account within the master group by passing the `account` parameter in the payload.

---

## Permission Levels (Roles)

Gemini uses **role-based access control** on API keys. Roles are assigned at key creation time in the web UI.

### Available Roles

| Role | Boolean Field | Description |
|------|---------------|-------------|
| **Trader** | `isTrader` | Can place, cancel, and manage orders; access trading history |
| **Auditor** | `isAuditor` | Read-only access; cannot place orders or move funds |
| **FundManager** | `isFundManager` | Can manage funds: deposits, withdrawals, transfers |
| **Administrator** | `isAccountAdmin` | Can create/rename sub-accounts (Master keys only) |

### Role Combinations

| Combination | Allowed? |
|-------------|----------|
| Trader only | Yes |
| Auditor only | Yes |
| FundManager only | Yes |
| Trader + FundManager | Yes |
| Auditor + Trader | NO — Auditor is mutually exclusive |
| Auditor + FundManager | NO — Auditor is mutually exclusive |
| Auditor + any other role | NO |
| Administrator + Trader/FundManager | Yes (Master key only) |

**Key Rule:** The Auditor role CANNOT be combined with any other role. All other roles can be combined freely.

### Default Role

When creating an API key, the **Trader role is assigned by default**.

---

### What Each Role Can Access

#### Trader Role

Required for:
- `POST /v1/order/new` — place orders
- `POST /v1/order/cancel` — cancel single order
- `POST /v1/order/cancel/all` — cancel all orders
- `POST /v1/order/cancel/session` — cancel session orders
- `POST /v1/order/status` — get order status
- `POST /v1/orders` — list active orders
- `POST /v1/orders/history` — order history
- `POST /v1/mytrades` — trade history
- `POST /v1/tradevolume` — trading volume
- `POST /v1/notionalvolume` — fee tier/volume
- `POST /v1/wrap/{symbol}` — wrap orders
- `POST /v1/balances` — account balances
- `POST /v1/notionalbalances/{currency}` — notional balances

#### Auditor Role (Read-Only)

Can access read-only endpoints including:
- Order status queries
- Balance queries
- Order history
- Trade history
- Volume/fee tier data
- Account detail

Cannot place orders, cannot transfer funds.

**Special constraint:** The FX Rate endpoint (`GET /v2/fxrate/{symbol}/{timestamp}`) requires specifically the **Auditor role**.

#### FundManager Role

Required for (fund movement operations):
- `POST /v1/withdraw/{currency}` — withdraw funds
- `POST /v1/deposit/{network}/newAddress` — create deposit addresses
- `POST /v1/account/transfer/{currency}` — transfer between sub-accounts
- `POST /v1/approvedAddresses/{network}/request` — add approved withdrawal address

Also has access to fund management read endpoints (balances, addresses, transfers).

#### Administrator Role (Master Keys Only)

Required for:
- `POST /v1/account/create` — create new sub-accounts
- `POST /v1/account/rename` — rename accounts
- `POST /v1/account/list` — list all accounts in group

---

## Check Current Key's Roles

**Method:** POST
**Path:** `/v1/roles`
**Auth Required:** Yes

Returns a JSON object indicating which roles the current key has:
```json
{
  "isAuditor": false,
  "isFundManager": true,
  "isTrader": true,
  "isAccountAdmin": false,
  "counterparty_id": "abc123"
}
```

---

## Session Management and Heartbeat

### Session-Based API Keys

API keys can optionally be configured as **session keys** with the "Requires Heartbeat" option enabled.

#### Heartbeat Behavior
- If heartbeat is enabled and the application does NOT send a heartbeat within **30 seconds**, the exchange will **automatically cancel all outstanding orders** for that session.
- This acts as a safety fail-safe for algorithmic trading systems.
- Heartbeat messages should be sent every **15 seconds maximum** to maintain the session.
- Multiple API keys per account are supported for independent session management.

---

## Rate Limits

### Public Endpoints (No Authentication)

| Limit | Value |
|-------|-------|
| Requests per minute | 120 |
| Recommended max rate | 1 request/second |

### Private Endpoints (Authenticated)

| Limit | Value |
|-------|-------|
| Requests per minute | 600 |
| Recommended max rate | 5 requests/second |

### Burst Mechanism

When the request rate exceeds the limit, Gemini provides a **burst queue**:
- Up to **5 additional requests** are queued with delayed processing.
- Any requests beyond the burst queue receive **HTTP 429 Too Many Requests**.
- The 429 persists until the rate drops below the threshold.

**Example:** Sending 20 rapid requests to a private endpoint:
- 10 processed immediately
- 5 queued (delayed processing)
- 5+ receive 429 responses

### Rate Limit Error Response

```json
{
  "result": "error",
  "reason": "Too Many Requests",
  "message": "Too Many Requests"
}
```
HTTP Status: `429`

### Order-Specific Rate Limits

NOT SEPARATELY DOCUMENTED. No documented special rate limit for order placement beyond the general private API limit of 600 req/min.

### Different Limits per Role

NOT DOCUMENTED. No documented differentiation in rate limits based on API key role (Trader vs Auditor vs FundManager).

---

## IP Restrictions

### Configuration

- IP restrictions are configured in the Gemini web UI **at API key creation time**.
- Two options:
  1. **Trusted IPs Only** — only requests from whitelisted IP addresses are accepted.
  2. **Unrestricted** — requests accepted from any IP address.

### Enforcement

- Gemini recommended and began enforcing that all Trading API keys have either explicit IP allowlists or be explicitly set as Unrestricted.
- Enforcement deadline announced: **June 30, 2025** — keys that have not been affirmed would have access blocked.

### Management via API

NOT AVAILABLE — IP whitelist management is done exclusively through the Gemini web UI. There is no API endpoint to add, remove, or view IP restrictions.

---

## OAuth (Alternative Authentication)

Gemini also supports **OAuth** as an alternative to API key authentication. The OAuth API is documented separately at `https://docs.gemini.com/rest/o-auth`.

OAuth scopes include `orders:create` for order placement. Full OAuth scope list is documented in the OAuth reference. This is separate from the API key role system.

---

## Sandbox Environment

- URL: `https://api.sandbox.gemini.com`
- Identical API surface as production.
- Separate API keys required (created at `exchange.sandbox.gemini.com`).
- No real funds involved.

---

## Sources

- [API Key Authentication — Gemini Crypto Exchange](https://docs.gemini.com/authentication/api-key)
- [Roles — Gemini Crypto Exchange](https://docs.gemini.com/roles)
- [Rate Limits — Gemini Crypto Exchange](https://docs.gemini.com/rate-limit)
- [Private API Invocation — Gemini WebSocket Overview](https://docs.gemini.com/websocket/overview/requests/private-api)
- [OAuth — REST API — Gemini Crypto Exchange](https://docs.gemini.com/rest/o-auth)
- [How to secure your API Keys with Trusted IPs — Gemini Support](https://support.gemini.com/hc/en-us/articles/37826759865115-How-to-secure-your-API-Keys-with-Trusted-IPs)
- [Account Administration — REST API — Gemini Crypto Exchange](https://docs.gemini.com/rest/account-administration)
