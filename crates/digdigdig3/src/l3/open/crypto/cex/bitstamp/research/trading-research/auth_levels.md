# Bitstamp Authentication, Rate Limits, and Testnet

---

## 1. API KEY PERMISSIONS

Bitstamp uses a fine-grained permission model. Each API key can be configured with specific scopes.

### 1.1 Permission Categories

| Permission | Scope | Required For |
|------------|-------|--------------|
| **Account Balance** | Read account balances | `get_balance()`, fee queries |
| **User Transactions** | Read trade/deposit/withdrawal history | `get_user_transactions()` |
| **Orders** | Place and cancel orders | All trading operations |
| **Withdrawals** | Initiate crypto/fiat withdrawals | Withdrawal endpoints |
| **Deposits** | View deposit addresses | Deposit address endpoints |
| **Transfers** | Sub-account transfers | `transfer_to_main()`, `transfer_from_main()` |

### 1.2 Recommended Permission Sets

**Trading bot (no withdrawals):**
- Account Balance: enabled
- User Transactions: enabled
- Orders: enabled
- Withdrawals: DISABLED
- Deposits: optional

**Portfolio tracker / read-only:**
- Account Balance: enabled
- User Transactions: enabled
- Orders: disabled
- Withdrawals: DISABLED
- Deposits: disabled

**Full access:**
- All permissions enabled (CAUTION: exposes withdrawal risk)

### 1.3 API Key Management
- Keys are created in the Bitstamp web UI under Settings > API Access
- Each key has: API Key (public), API Secret (private), Customer ID (numeric)
- Multiple keys can be created with different permission sets
- Kill-switch endpoint: `POST /api/v2/revoke_all_api_keys/` — disables ALL keys on the account (security panic button, added 2023)

---

## 2. AUTHENTICATION MECHANISM

Bitstamp uses a custom header-based HMAC-SHA256 authentication (v2 auth).

### 2.1 Required Headers

| Header | Value | Description |
|--------|-------|-------------|
| `X-Auth` | `"BITSTAMP " + api_key` | Authentication identifier (note: space after BITSTAMP) |
| `X-Auth-Signature` | HMAC-SHA256 hex string | Request signature (see construction below) |
| `X-Auth-Nonce` | UUID v4 string | Random nonce, lowercase, 36 chars, e.g. `"f93c979d-b00d-43a9-9b9c-fd4cd9547fa6"` |
| `X-Auth-Timestamp` | UTC milliseconds as string | e.g. `"1567755304968"` |
| `X-Auth-Version` | `"v2"` | Auth version |
| `Content-Type` | `"application/x-www-form-urlencoded"` | For POST with body; OMIT if no body |

### 2.2 Signature Construction

The string to sign is a concatenation of the following components (no separator, raw concatenation):

```
"BITSTAMP " + api_key
+ HTTP_METHOD        (uppercase, e.g. "POST", "GET")
+ host               (e.g. "www.bitstamp.net")
+ path               (e.g. "/api/v2/balance/")
+ query_string       (empty string if no query params)
+ content_type       (e.g. "application/x-www-form-urlencoded", or "" if no body)
+ nonce              (the UUID value used in X-Auth-Nonce)
+ timestamp          (the millisecond UTC timestamp used in X-Auth-Timestamp)
+ "v2"               (version string)
+ request_body       (URL-encoded body string, or "" if no body)
```

**HMAC-SHA256:** Sign the concatenated string with the API secret key, output as uppercase hex.

**Example string to sign (conceptual):**
```
BITSTAMP {api_key}POSTwww.bitstamp.net/api/v2/balance/application/x-www-form-urlencodedF93C979D-B00D-43A9-9B9C-FD4CD9547FA615677553049680v2
```

**Rust implementation sketch:**
```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

fn build_signature(
    api_key: &str,
    api_secret: &str,
    method: &str,       // "POST"
    host: &str,         // "www.bitstamp.net"
    path: &str,         // "/api/v2/balance/"
    query: &str,        // "" or "foo=bar"
    content_type: &str, // "application/x-www-form-urlencoded" or ""
    nonce: &str,        // UUID v4 lowercase
    timestamp: &str,    // millis since epoch as string
    body: &str,         // URL-encoded body or ""
) -> String {
    let msg = format!(
        "BITSTAMP {api_key}{method}{host}{path}{query}{content_type}{nonce}{timestamp}v2{body}"
    );
    let mut mac = Hmac::<Sha256>::new_from_slice(api_secret.as_bytes())
        .expect("HMAC key error");
    mac.update(msg.as_bytes());
    let result = mac.finalize().into_bytes();
    hex::encode_upper(result) // uppercase hex
}
```

### 2.3 Nonce Rules
- Must be a UUID v4 format: `xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx`
- Lowercase
- Exactly 36 characters
- Each nonce is valid ONCE within a 150-second window
- Re-using a nonce within 150 seconds will be rejected

### 2.4 Timestamp Rules
- UTC milliseconds since Unix epoch
- Request is rejected if timestamp differs from server time by more than 150 seconds
- Generate fresh timestamp for each request

### 2.5 Content-Type Special Case
- If the request has NO body (e.g., `POST /api/v2/open_orders/all/` with no params), the `Content-Type` header MUST be OMITTED from both the request headers AND the signature string
- Including `Content-Type` in the signature when omitted from headers (or vice versa) causes auth failure

### 2.6 Legacy Auth (v1 — DO NOT USE)
The old authentication passed `key`, `signature`, and `nonce` as form body parameters. This is deprecated. Use header-based v2 auth exclusively.

---

## 3. RATE LIMITS

### 3.1 Standard Limits

| Limit Type | Value |
|------------|-------|
| Requests per second | **400 req/sec** |
| Requests per 10 minutes | **10,000 requests** |

These are account-level limits (not per-endpoint).

### 3.2 Rate Limit Errors
| Error Code | Meaning |
|------------|---------|
| `400.002` | Rate limit exceeded — slow down requests |

### 3.3 Limit Increase
- Higher limits available via bespoke agreement with Bitstamp (enterprise/institutional)
- Contact Bitstamp support to negotiate increased limits

### 3.4 Third-Party Observed Limits
- Some third-party wrappers enforce self-imposed 600 requests/10 minutes as conservative safety
- Official documented limit is 10,000 per 10 minutes (400/sec)

### 3.5 Caching Notes
- `open_orders` endpoints: results are cached server-side for approximately 10 seconds
- Do not poll open orders faster than every 10 seconds

---

## 4. TESTNET / SANDBOX

### 4.1 Official Sandbox
Bitstamp provides a **sandbox environment** at:
```
https://sandbox.bitstamp.net
```

The sandbox mirrors the production API structure. Example endpoint:
```
POST https://sandbox.bitstamp.net/api/v2/balance/
POST https://sandbox.bitstamp.net/api/v2/fees/trading/
```

- Requires separate sandbox API keys (generated within the sandbox environment)
- Not tied to real funds
- Latency and liquidity differ from production
- Use for testing order flows and error handling logic

### 4.2 No Public Access Testnet
- Historically (pre-2017), Bitstamp had no sandbox
- A sandbox was confirmed to exist in modern documentation (referenced in fee endpoint docs showing sandbox URLs)
- Access typically requires registering a sandbox account
- If sandbox access is unavailable, the standard approach is testing with minimal real amounts

### 4.3 V5 Trait Design Implication
- `base_url` should be configurable: `https://www.bitstamp.net` (prod) or `https://sandbox.bitstamp.net` (test)
- Auth mechanism is identical between prod and sandbox

---

## 5. API VERSIONING

| Version | Status | Notes |
|---------|--------|-------|
| v1 (`/api/`) | Deprecated | Legacy form-based auth with nonce in body |
| v2 (`/api/v2/`) | **Current** | Header-based HMAC-SHA256 auth |

Always use `/api/v2/` endpoints.

---

## 6. RECENT API CHANGES (2023–2025)

| Date | Change |
|------|--------|
| Nov 2023 | `client_order_id` uniqueness no longer enforced (duplicates allowed) |
| 2023 | `currency_pair` field deprecated in order responses; replaced by `market` |
| 2023 | `revoke_all_api_keys` kill-switch endpoint added |
| 2023 | Travel Rule compliance endpoints added |
| 2023 | GTD order expiration semantics clarified |
| 2023 | Extended error responses for cancellations |
| Apr 2025 | Stop Market Buy Orders discontinued |
| Apr 2025 | Trailing Stop Orders (all types) discontinued |
| May 2025 | Existing Stop Market and Trailing Stop orders auto-closed |

---

## 7. SUMMARY FOR V5 RUST IMPLEMENTATION

```rust
// Auth config struct
pub struct BitstampAuth {
    pub api_key: String,
    pub api_secret: String,
    pub customer_id: String,  // numeric, from account settings
}

// Headers to set on every private request
// X-Auth: "BITSTAMP {api_key}"
// X-Auth-Signature: uppercase hex HMAC-SHA256
// X-Auth-Nonce: UUID v4 lowercase
// X-Auth-Timestamp: current UTC millis as string
// X-Auth-Version: "v2"
// Content-Type: "application/x-www-form-urlencoded" (only if body is non-empty)

// Rate limit: 400 req/sec, 10,000 per 10 min
// No burst headers returned — must track client-side

// Testnet base URL: https://sandbox.bitstamp.net
// Prod base URL:    https://www.bitstamp.net
```
