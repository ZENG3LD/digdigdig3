# Coinbase Advanced Trade API — Authentication & Rate Limits

---

## 1. API KEY TYPES

### 1.1 CDP API Keys (Current Standard)

Coinbase migrated to **CDP (Coinbase Developer Platform) API Keys** as the mandatory authentication method.
Legacy API keys were **deprecated and removed effective June 10, 2024**.

**CDP Key Format:**
```
Key Name:    "organizations/{org_id}/apiKeys/{key_id}"
Private Key: "-----BEGIN EC PRIVATE KEY-----\n...\n-----END EC PRIVATE KEY-----\n"
```

Example:
```
key_name    = "organizations/3b0ef0bc-abcd-1234-abcd-abc123456789/apiKeys/a1b2c3d4-1234-5678-abcd-ef0123456789"
private_key = "-----BEGIN EC PRIVATE KEY-----\nMHQCAQ...\n-----END EC PRIVATE KEY-----\n"
```

The private key is an ECDSA EC private key (P-256 / secp256r1 curve), provided as PEM-encoded format.

**Important**: Coinbase App APIs support **only ES256 (ECDSA)**. Ed25519 keys are NOT supported for Advanced Trade. This differs from the newer CDP wallet APIs which prefer Ed25519.

### 1.2 Legacy API Keys (REMOVED)

Legacy Coinbase Pro / Coinbase Advanced keys used HMAC-SHA256 signing. These are completely removed as of June 2024. No new keys can be created; existing ones no longer work.

---

## 2. JWT AUTHENTICATION

Every authenticated request requires a freshly generated JWT token. Tokens expire after **2 minutes** and must not be reused.

### 2.1 JWT Structure

A JWT consists of three base64url-encoded segments: `header.payload.signature`

**Header:**
```json
{
  "alg": "ES256",
  "kid": "organizations/{org_id}/apiKeys/{key_id}",
  "nonce": "a3f9c2d1b8e047f6a9c2d3e4b5a6f7d8",
  "typ": "JWT"
}
```

| Field | Value | Description |
|---|---|---|
| `alg` | `"ES256"` | ECDSA with SHA-256 — fixed, do not change |
| `kid` | your key name | Identifies which CDP API key is signing |
| `nonce` | 32-char hex string | Random, prevents replay attacks |
| `typ` | `"JWT"` | Always "JWT" |

**Payload:**
```json
{
  "sub": "organizations/{org_id}/apiKeys/{key_id}",
  "iss": "cdp",
  "aud": ["cdp_service"],
  "nbf": 1716789000,
  "exp": 1716789120,
  "uri": "GET api.coinbase.com/api/v3/brokerage/accounts"
}
```

| Field | Value | Description |
|---|---|---|
| `sub` | your key name | Subject — same as `kid` in header |
| `iss` | `"cdp"` | Issuer — always this exact string |
| `aud` | `["cdp_service"]` | Audience — always this exact array |
| `nbf` | Unix timestamp (now) | Not valid before this time |
| `exp` | Unix timestamp (now + 120) | Expires 2 minutes from `nbf` |
| `uri` | `"{METHOD} {HOST}{PATH}"` | Scoped to specific request |

**URI Format:**
```
"GET api.coinbase.com/api/v3/brokerage/accounts"
"POST api.coinbase.com/api/v3/brokerage/orders"
```
- No `https://` prefix
- No query string
- Include HTTP method, space, host, full path

### 2.2 Nonce Generation

Generate a 32-character lowercase hexadecimal string:

```rust
// Rust
use rand::Rng;
let nonce: String = (0..16)
    .map(|_| format!("{:02x}", rand::thread_rng().gen::<u8>()))
    .collect();
// Result: "a3f9c2d1b8e047f6a9c2d3e4b5a6f7d8"
```

```python
# Python
import secrets
nonce = secrets.token_hex(16)  # 32 hex chars
```

### 2.3 Signing the JWT

1. Encode header and payload as base64url JSON strings
2. Concatenate as `{encoded_header}.{encoded_payload}`
3. Sign with EC private key using ES256 (ECDSA with SHA-256)
4. Append base64url-encoded signature

```rust
// Rust (using jsonwebtoken crate)
use jsonwebtoken::{encode, Header, EncodingKey, Algorithm};

let mut header = Header::new(Algorithm::ES256);
header.kid = Some(key_name.to_string());
header.typ = Some("JWT".to_string());
// Note: nonce requires custom header field — may need jwt-simple or manual construction

let encoding_key = EncodingKey::from_ec_pem(private_key_pem.as_bytes())?;
let token = encode(&header, &claims, &encoding_key)?;
```

### 2.4 Using the JWT in Requests

Add to HTTP Authorization header:

```
Authorization: Bearer eyJhbGciOiJFUzI1NiIsImtpZCI6...
```

For REST:
```http
GET /api/v3/brokerage/accounts HTTP/1.1
Host: api.coinbase.com
Authorization: Bearer {JWT_TOKEN}
Content-Type: application/json
```

For WebSocket, send JWT as part of the subscription message (not in the HTTP upgrade headers).

### 2.5 Implementation Notes for Rust

- Generate a new JWT for **each HTTP request** (tokens are request-scoped)
- The `uri` claim binds the token to a specific method + path
- Must generate before constructing the request (need the method + path first)
- 2-minute expiry means tokens should not be cached or reused across calls
- Private key PEM string contains literal `\n` from Coinbase export — replace with actual newlines before parsing

---

## 3. API KEY PERMISSIONS

CDP API keys have granular permissions set at creation time:

| Permission | Scope | Allows |
|---|---|---|
| `view` | Read-only | GET endpoints: accounts, orders, fills, products, portfolios |
| `trade` | Trading | POST create/cancel/edit orders |
| `transfer` | Fund movement | Move funds between portfolios, payment methods |

Keys should be created with **minimum necessary permissions**. A trading bot typically needs `view` + `trade`. If moving funds between portfolios (e.g. INTX margin top-up), also needs `transfer`.

**IP Allowlisting**: CDP keys support IP allowlisting for additional security. Recommended for production bots.

---

## 4. RATE LIMITS

### 4.1 REST API Rate Limits

| Category | Limit | Scope |
|---|---|---|
| Private endpoints | **30 requests/second** | Per user (API key) |
| Public endpoints | **10 requests/second** | Per IP address |

Private endpoints = any endpoint requiring authentication (orders, accounts, fills, etc.)
Public endpoints = product listings, order book snapshots, candles without auth

### 4.2 Fills Endpoint — Custom Limit

The fills endpoint (`GET /api/v3/brokerage/orders/historical/fills`) has a **lower rate limit** than standard private endpoints. Exact limit is not publicly documented but is more restrictive — treat it as approximately **5 requests/second** in practice.

### 4.3 Public Endpoint Caching

All public endpoints have a **1-second cache** enabled by default.

To bypass the cache:
```http
cache-control: no-cache
```
Or use WebSocket streams for real-time data without caching.

### 4.4 Rate Limit Response

When exceeded, the API returns:
```
HTTP 429 Too Many Requests
```

Response body:
```json
{
  "error": "RATE_LIMIT_EXCEEDED",
  "message": "Too many requests",
  "error_details": "",
  "preview_failure_reason": "",
  "new_order_failure_reason": ""
}
```

No `Retry-After` header is consistently included. Implement exponential backoff starting at 1 second.

### 4.5 WebSocket Rate Limits

| Category | Limit |
|---|---|
| Authenticated connections | 750 messages/second per IP |
| Unauthenticated messages | 8 messages/second per IP |

Exceeding WebSocket limits results in connection termination.

---

## 5. SANDBOX / TESTNET ENVIRONMENT

### 5.1 Sandbox Base URL

```
https://api-sandbox.coinbase.com/api/v3/brokerage/
```

vs production:
```
https://api.coinbase.com/api/v3/brokerage/
```

### 5.2 Sandbox Authentication

The sandbox uniquely supports **unauthenticated requests** — no JWT token needed. This allows quick testing without needing valid CDP keys.

However, production CDP keys can also be used with the sandbox URL.

### 5.3 Available Endpoints in Sandbox

Only a subset is available:
- Accounts (list, get)
- Orders (create, cancel, edit, preview, list, get)
- Fills (list)
- Portfolios (list, get)
- Perpetuals (some endpoints)

Product market data endpoints and WebSocket are **not available** in sandbox.

### 5.4 Static Mock Responses

Sandbox returns **static, pre-defined responses** — not live market data. Prices and balances are hardcoded.

### 5.5 Scenario Testing via Custom Header

To test specific error scenarios, pass the `X-Sandbox` header:

```http
X-Sandbox: PostOrder_insufficient_fund
```

Common scenario values:
- `PostOrder_insufficient_fund` — triggers INSUFFICIENT_FUND failure
- Other scenarios vary; check current Coinbase sandbox docs for the full list

### 5.6 Separate Sandbox API Keys

For authenticated sandbox testing, create sandbox-specific API keys at:
`https://portal.cdp.coinbase.com/` (select Sandbox environment)

Sandbox keys cannot be used with the production API and vice versa.

---

## 6. COMPLETE RUST IMPLEMENTATION SKETCH

```rust
use std::time::{SystemTime, UNIX_EPOCH};

struct CoinbaseAuth {
    key_name: String,     // "organizations/{org}/apiKeys/{id}"
    private_key_pem: String,  // EC PRIVATE KEY PEM
}

struct JwtClaims {
    sub: String,          // = key_name
    iss: String,          // = "cdp"
    aud: Vec<String>,     // = ["cdp_service"]
    nbf: u64,             // current unix timestamp
    exp: u64,             // nbf + 120
    uri: String,          // "{METHOD} api.coinbase.com{path}"
}

impl CoinbaseAuth {
    fn generate_jwt(&self, method: &str, path: &str) -> Result<String, Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let nonce = generate_hex_nonce(16); // 32 hex chars

        let uri = format!("{} api.coinbase.com{}", method, path);

        let claims = JwtClaims {
            sub: self.key_name.clone(),
            iss: "cdp".to_string(),
            aud: vec!["cdp_service".to_string()],
            nbf: now,
            exp: now + 120,
            uri,
        };

        // Construct JWT header with nonce (requires custom JWT library or manual assembly)
        // Header: {"alg":"ES256","kid":"{key_name}","nonce":"{nonce}","typ":"JWT"}
        // Sign with ES256 using EC private key
        // Return base64url(header).base64url(payload).base64url(signature)
        todo!()
    }
}
```

---

## 7. QUICK REFERENCE — AUTH CHECKLIST

| Step | Detail |
|---|---|
| Algorithm | ES256 (ECDSA P-256 / SHA-256) only — NOT Ed25519, NOT HMAC |
| Key name format | `"organizations/{org_id}/apiKeys/{key_id}"` |
| Private key format | PEM: `-----BEGIN EC PRIVATE KEY-----` |
| JWT expiry | 2 minutes (120 seconds) |
| Nonce | 32 hex chars, fresh per request |
| URI claim format | `"METHOD api.coinbase.com/path"` (no https://) |
| Header field | `Authorization: Bearer {jwt}` |
| Per-request JWT | YES — generate a new token for each HTTP call |
| Legacy HMAC keys | REMOVED June 2024 — do not implement |
| Sandbox URL | `https://api-sandbox.coinbase.com/api/v3/brokerage/` |
| Sandbox auth | Optional (requests work unauthenticated) |

---

## Sources

- [JWT Authentication Guide](https://docs.cdp.coinbase.com/get-started/authentication/jwt-authentication)
- [Coinbase App API Key Authentication](https://docs.cdp.coinbase.com/coinbase-app/authentication-authorization/api-key-authentication)
- [Advanced Trade REST API Auth](https://docs.cdp.coinbase.com/advanced-trade/docs/rest-api-auth)
- [Advanced Trade WebSocket Auth](https://docs.cdp.coinbase.com/advanced-trade/docs/ws-auth)
- [Advanced Trade WebSocket Rate Limits](https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/websocket/websocket-rate-limits)
- [Advanced Trade API Sandbox](https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/sandbox)
- [CDP API Keys Overview](https://docs.cdp.coinbase.com/get-started/docs/cdp-api-keys/)
- [CCXT Coinbase Advanced Key Format Issue](https://github.com/ccxt/ccxt/issues/21226)
