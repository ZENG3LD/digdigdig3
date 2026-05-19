# Coinbase Advanced Trade API Authentication Research

**Research Date**: 2026-01-20
**API Documentation Version**: 2026 (Latest)

---

## 1. Required Headers for Authenticated Requests

All private REST requests to Coinbase Advanced Trade API **MUST** include the following header:

| Header Name | Description | Example Value |
|-------------|-------------|---------------|
| `Authorization` | Bearer token with JWT | `"Bearer eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6Im9yZ2FuaXphdGlvbnMve29yZ19pZH0vYXBpS2V5cy97a2V5X2lkfSIsIm5vbmNlIjoiYzU3ZTM5YjE4MTU5NDRkYjk5ODk4ZTUzMDg1YjhkYTIifQ..."` |

**Note**: Unlike KuCoin which uses multiple headers (KC-API-KEY, KC-API-SIGN, etc.), Coinbase uses a **single Authorization header** with a JWT token.

---

## 2. JWT Structure

### 2.1 What is a JWT?

A JSON Web Token (JWT) consists of three parts separated by dots:
```
header.payload.signature
```

Example:
```
eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6Im9yZ2FuaXphdGlvbnMve29yZ31fYXBpS2V5cy97a2V5fSIsIm5vbmNlIjoiYzU3ZTM5YjE4MTU5NDRkYjk5ODk4ZTUzMDg1YjhkYTIifQ.eyJzdWIiOiJvcmdhbml6YXRpb25zL3tvcmdfaWR9L2FwaUtleXMve2tleV9pZH0iLCJpc3MiOiJjZHAiLCJuYmYiOjE3MDY5ODY2MzAsImV4cCI6MTcwNjk4Njc1MCwidXJpIjoiR0VUIGFwaS5jb2luYmFzZS5jb20vYXBpL3YzL2Jyb2tlcmFnZS9hY2NvdW50cyJ9.sig
```

### 2.2 JWT Header

The JWT header **MUST** include these fields:

```json
{
  "alg": "ES256",
  "typ": "JWT",
  "kid": "organizations/{org_id}/apiKeys/{key_id}",
  "nonce": "c57e39b1815944db99898e53085b8da2"
}
```

**Header Fields:**
- `alg` (string): **"ES256"** - ECDSA with P-256 curve (REQUIRED)
- `typ` (string): **"JWT"** - Token type (OPTIONAL but recommended)
- `kid` (string): **Your API key name** in format `organizations/{org_id}/apiKeys/{key_id}` (REQUIRED)
- `nonce` (string): **Random hexadecimal string** for replay attack prevention (REQUIRED)

**Important**:
- Only **ES256** (ECDSA with P-256 curve) is supported
- **Ed25519 (EdDSA) keys are NOT supported**

### 2.3 JWT Payload

The JWT payload **MUST** include these claims:

```json
{
  "sub": "organizations/{org_id}/apiKeys/{key_id}",
  "iss": "cdp",
  "nbf": 1706986630,
  "exp": 1706986750,
  "uri": "GET api.coinbase.com/api/v3/brokerage/accounts"
}
```

**Payload Fields:**
- `sub` (string): **Subject** - Your API key name (same as `kid`) (REQUIRED)
- `iss` (string): **Issuer** - Must be **"cdp"** (REQUIRED)
- `nbf` (integer): **Not Before** - Current Unix timestamp in seconds (REQUIRED)
- `exp` (integer): **Expiration** - Current timestamp + 120 seconds (REQUIRED)
- `uri` (string): **URI** - Formatted request specification (REQUIRED)

### 2.4 URI Field Format

The `uri` field **MUST** be constructed as:
```
"{HTTP_METHOD} {HOST}{PATH}"
```

**Components:**
- `HTTP_METHOD`: Uppercase HTTP method (GET, POST, DELETE, etc.)
- `HOST`: API hostname **without** protocol (e.g., `api.coinbase.com`)
- `PATH`: Request path including query string (e.g., `/api/v3/brokerage/accounts`)

**Examples:**

| Request | URI Field Value |
|---------|-----------------|
| `GET https://api.coinbase.com/api/v3/brokerage/accounts` | `"GET api.coinbase.com/api/v3/brokerage/accounts"` |
| `POST https://api.coinbase.com/api/v3/brokerage/orders` | `"POST api.coinbase.com/api/v3/brokerage/orders"` |
| `GET https://api.coinbase.com/api/v3/brokerage/products/BTC-USD` | `"GET api.coinbase.com/api/v3/brokerage/products/BTC-USD"` |

**Critical Notes:**
- **NO** `https://` protocol prefix
- **NO** port number (use default ports)
- **INCLUDE** full path after hostname
- For GET requests with query parameters, the URI does **NOT** include query string (only path)

---

## 3. Signature Algorithm

### 3.1 Algorithm

**ES256** (ECDSA using P-256 curve and SHA-256 hash)

This is a digital signature algorithm that uses:
- **Elliptic Curve**: NIST P-256 (also known as secp256r1 or prime256v1)
- **Hash Function**: SHA-256
- **Encoding**: Standard JWT encoding (base64url)

### 3.2 Key Requirements

**API Key Creation:**
1. When creating API keys in Coinbase Developer Platform, you receive:
   - **API Key Name** (e.g., `organizations/abc123/apiKeys/def456`)
   - **Private Key** (EC private key in PEM format)
2. The private key is shown **only once** - save it securely
3. Private key format example:
   ```
   -----BEGIN EC PRIVATE KEY-----
   MHcCAQEEIBKhX...base64...kpbAgEA
   -----END EC PRIVATE KEY-----
   ```

**Key Format:**
- Must be ECDSA (Elliptic Curve)
- Must use P-256 curve
- PEM-encoded
- Newlines must be preserved in the private key string

### 3.3 Signing Process

**Step-by-step:**

1. **Create JWT Header**:
   ```json
   {
     "alg": "ES256",
     "typ": "JWT",
     "kid": "organizations/{org_id}/apiKeys/{key_id}",
     "nonce": "{random_hex_16_bytes}"
   }
   ```

2. **Create JWT Payload**:
   ```json
   {
     "sub": "organizations/{org_id}/apiKeys/{key_id}",
     "iss": "cdp",
     "nbf": {current_unix_timestamp},
     "exp": {current_unix_timestamp + 120},
     "uri": "{METHOD} api.coinbase.com{path}"
   }
   ```

3. **Encode Header and Payload**:
   - Base64URL encode header → `header_b64`
   - Base64URL encode payload → `payload_b64`
   - Create signing input: `{header_b64}.{payload_b64}`

4. **Sign**:
   - Load EC private key from PEM format
   - Sign the signing input using ES256 algorithm
   - Encode signature with Base64URL → `signature_b64`

5. **Construct JWT**:
   ```
   {header_b64}.{payload_b64}.{signature_b64}
   ```

6. **Set Authorization Header**:
   ```
   Authorization: Bearer {jwt}
   ```

### 3.4 Python Example (Official)

```python
import jwt
import time
import secrets

# Load private key
with open('path/to/ec_private_key.pem', 'r') as f:
    private_key = f.read()

# Generate JWT
def build_jwt(service, uri):
    key_name = "organizations/{org_id}/apiKeys/{key_id}"

    # Header
    header = {
        "alg": "ES256",
        "typ": "JWT",
        "kid": key_name,
        "nonce": secrets.token_hex(16)  # 16 bytes = 32 hex chars
    }

    # Payload
    current_time = int(time.time())
    payload = {
        "sub": key_name,
        "iss": "cdp",
        "nbf": current_time,
        "exp": current_time + 120,  # 2 minutes
        "uri": f"{service} api.coinbase.com{uri}"
    }

    # Sign and encode
    token = jwt.encode(payload, private_key, algorithm="ES256", headers=header)
    return token

# Usage
jwt_token = build_jwt("GET", "/api/v3/brokerage/accounts")
headers = {
    "Authorization": f"Bearer {jwt_token}"
}
```

---

## 4. Timestamp and Expiration

### 4.1 Timestamp Format

**Format**: Unix timestamp in **seconds** (not milliseconds)

**Example**: `1706986630` (represents 2024-02-03T15:37:10Z)

### 4.2 Token Validity

- **Duration**: 2 minutes (120 seconds)
- **nbf** (Not Before): Current time
- **exp** (Expiration): Current time + 120 seconds

**Important**:
- Generate a **new JWT for each request**
- JWTs expire after 2 minutes
- Reusing an expired JWT will result in 401 Unauthorized

### 4.3 Time Sync Requirements

- Server-client time difference must be within **30 seconds**
- If your system clock is off by more than 30 seconds, requests will fail
- Use `GET /api/v3/brokerage/time` to check server time

---

## 5. Nonce Generation

### 5.1 Purpose

The `nonce` (number used once) prevents replay attacks by ensuring each JWT is unique.

### 5.2 Format

- **Type**: Random hexadecimal string
- **Length**: 32 characters (16 bytes)
- **Example**: `"c57e39b1815944db99898e53085b8da2"`

### 5.3 Generation Methods

**Python**:
```python
import secrets
nonce = secrets.token_hex(16)  # 16 bytes = 32 hex chars
```

**JavaScript**:
```javascript
const crypto = require('crypto');
const nonce = crypto.randomBytes(16).toString('hex');
```

**Rust**:
```rust
use rand::Rng;
let nonce: String = rand::thread_rng()
    .sample_iter(&rand::distributions::Alphanumeric)
    .take(32)
    .map(char::from)
    .collect();
```

---

## 6. Differences from KuCoin Authentication

### 6.1 Authentication Method

| Feature | KuCoin | Coinbase |
|---------|--------|----------|
| **Method** | HMAC-SHA256 | JWT (ES256) |
| **Headers** | 6 headers (KC-API-KEY, KC-API-SIGN, KC-API-TIMESTAMP, KC-API-PASSPHRASE, KC-API-KEY-VERSION, Content-Type) | 1 header (Authorization: Bearer {jwt}) |
| **Signature Algorithm** | HMAC-SHA256 | ECDSA P-256 (ES256) |
| **Key Type** | API Secret (symmetric) | EC Private Key (asymmetric) |
| **Encoding** | Base64 | Base64URL (JWT standard) |
| **Passphrase** | Required (encrypted) | Not used |
| **Timestamp Format** | Milliseconds | Seconds |

### 6.2 Signature String Format

**KuCoin**:
```
{timestamp}{method}{endpoint}{body}
```
Example: `1547015186532GET/api/v1/accounts`

**Coinbase**:
```
JWT with uri field: "{METHOD} {HOST}{PATH}"
```
Example URI: `GET api.coinbase.com/api/v3/brokerage/accounts`

### 6.3 Query String Handling

**KuCoin**:
- For GET/DELETE: Include query string in endpoint
- Example: `/api/v1/accounts?currency=BTC`

**Coinbase**:
- Query string **NOT** included in URI field
- Example URI: `GET api.coinbase.com/api/v3/brokerage/accounts` (no `?limit=250`)

### 6.4 Body Handling

**KuCoin**:
- Include full JSON body in signature string

**Coinbase**:
- Body **NOT** included in URI field
- Signature is part of JWT structure, not direct body signature

---

## 7. API Key Permissions

### 7.1 Permission Types

Coinbase API keys have three permission levels:

| Permission | Description |
|------------|-------------|
| **view** | Read-only access (accounts, orders, fills) |
| **trade** | Create, cancel, edit orders |
| **transfer** | Move funds between portfolios/accounts |

### 7.2 Permission Scoping

- Create separate API keys for different purposes
- Use "view" for monitoring, "trade" for trading bots
- Never use "transfer" permission unless necessary

---

## 8. Error Responses

### 8.1 Authentication Failures

**Invalid Signature**:
```json
{
  "error": "invalid_signature",
  "message": "Invalid JWT signature"
}
```
HTTP Status: `401 Unauthorized`

**Expired Token**:
```json
{
  "error": "token_expired",
  "message": "JWT token has expired"
}
```
HTTP Status: `401 Unauthorized`

**Invalid Key**:
```json
{
  "error": "invalid_api_key",
  "message": "API key not found"
}
```
HTTP Status: `401 Unauthorized`

### 8.2 Time Sync Issues

If server time and client time differ by more than 30 seconds:
```json
{
  "error": "invalid_token",
  "message": "Token nbf/exp out of acceptable range"
}
```

**Solution**:
1. Sync your system clock with NTP
2. Check server time: `GET /api/v3/brokerage/time`
3. Adjust your timestamp generation

---

## 9. Implementation Checklist

### 9.1 Required Components

- [ ] Load EC private key from PEM file
- [ ] Generate random 16-byte nonce for each request
- [ ] Build JWT header with `kid` and `nonce`
- [ ] Build JWT payload with correct claims
- [ ] Format URI field correctly (method + host + path)
- [ ] Sign JWT with ES256 algorithm
- [ ] Set Authorization header with Bearer token
- [ ] Handle JWT expiration (generate new JWT per request)

### 9.2 Testing Steps

1. **Test Server Time**: Call `GET /api/v3/brokerage/time`
2. **Test Public Endpoints**: Call `GET /market/products` (no auth)
3. **Test Authentication**: Call `GET /api/v3/brokerage/accounts` with JWT
4. **Test Token Expiration**: Reuse JWT after 2 minutes (should fail)
5. **Test Permissions**: Try trading endpoint with "view" only key (should fail)

---

## 10. Summary

### 10.1 Key Takeaways

1. **JWT-based Authentication**: Use ES256 (ECDSA P-256) to sign JWTs
2. **Single Header**: Only `Authorization: Bearer {jwt}` required
3. **2-Minute Validity**: Generate new JWT for each request
4. **No Passphrase**: Unlike KuCoin, no passphrase encryption needed
5. **URI Format**: `{METHOD} {HOST}{PATH}` (no protocol, no query string)
6. **Nonce Required**: Random 16-byte hex string in header
7. **Timestamp**: Unix seconds (not milliseconds)
8. **Time Sync**: Must be within 30 seconds of server

### 10.2 Common Pitfalls

- ❌ Using Ed25519 keys instead of ECDSA P-256
- ❌ Including `https://` in URI field
- ❌ Including query string in URI field
- ❌ Using milliseconds for nbf/exp (should be seconds)
- ❌ Reusing JWTs across multiple requests
- ❌ Not preserving newlines in PEM private key
- ❌ Using timestamp in milliseconds instead of seconds

### 10.3 Official Documentation Sources

- **Primary**: https://docs.cdp.coinbase.com/coinbase-app/authentication-authorization/api-key-authentication
- **Advanced Trade Auth**: https://docs.cdp.coinbase.com/advanced-trade/docs/rest-api-auth
- **WebSocket Auth**: https://docs.cdp.coinbase.com/advanced-trade/docs/ws-auth
- **Legacy Auth** (deprecated): https://docs.cdp.coinbase.com/advanced-trade/docs/rest-api-auth-legacy

---

## Sources

- [Coinbase App API Key Authentication](https://docs.cdp.coinbase.com/coinbase-app/authentication-authorization/api-key-authentication)
- [Advanced Trade API Authentication](https://docs.cdp.coinbase.com/advanced-trade/docs/rest-api-auth)
- [Advanced Trade WebSocket Authentication](https://docs.cdp.coinbase.com/advanced-trade/docs/ws-auth)
- [Coinbase Advanced Python SDK - Authentication](https://coinbase.github.io/coinbase-advanced-py/jwt_generator.html)
- [Coinbase Advanced Python SDK - GitHub](https://github.com/coinbase/coinbase-advanced-py)
- [Advanced Trade API Legacy Key Authentication](https://docs.cdp.coinbase.com/advanced-trade/docs/rest-api-auth-legacy)

---

**Research completed**: 2026-01-20
**Implementation note**: Coinbase uses JWT (ES256) authentication, fundamentally different from KuCoin's HMAC-SHA256 approach. Requires EC private key in PEM format and JWT library support.
