# Upbit API Authentication Research

**Research Date**: 2026-01-20
**API Documentation Version**: 2026 (Latest)

---

## 1. Authentication Overview

Upbit uses **JWT (JSON Web Token)** for authenticating private API requests. Unlike traditional HMAC-based signatures, Upbit requires generating a complete JWT token that includes request metadata and a cryptographic signature.

### Key Characteristics

- **Algorithm**: HS512 (HMAC with SHA-512)
- **Encoding**: Base64 for token components
- **Token Format**: `{header}.{payload}.{signature}`
- **Header Type**: Bearer token in `Authorization` header
- **No Base64 Decoding**: Secret Key used as-is (not Base64-decoded)

---

## 2. Required Headers for Authenticated Requests

All private REST requests to Upbit API **MUST** include the following header:

| Header Name | Description | Example Value |
|-------------|-------------|---------------|
| `Authorization` | Bearer token with JWT | `Bearer eyJhbGciOiJIUzUxMiIsInR5cCI6IkpXVCJ9...` |
| `Content-Type` | Request/response format | `application/json; charset=utf-8` |

**Example**:
```
Authorization: Bearer eyJhbGciOiJIUzUxMiIsInR5cCI6IkpXVCJ9.eyJhY2Nlc3Nfa2V5IjoiYWJjZGVmIiwibm9uY2UiOiIxMjM0LTU2NzgiLCJxdWVyeV9oYXNoIjoiYWJjZGVmMTIzNDU2IiwicXVlcnlfaGFzaF9hbGciOiJTSEE1MTIifQ.signature
Content-Type: application/json; charset=utf-8
```

---

## 3. JWT Token Structure

A JWT consists of three Base64URL-encoded parts separated by dots (`.`):

```
{header}.{payload}.{signature}
```

### 3.1 Header

Specifies the signing algorithm and token type.

**Structure**:
```json
{
  "alg": "HS512",
  "typ": "JWT"
}
```

**Fields**:
- `alg` (string): Algorithm for signature generation
  - **Recommended**: `"HS512"` (HMAC with SHA-512)
  - Also supported: `"HS256"` (HMAC with SHA-256)
- `typ` (string): Token type, always `"JWT"`

**Base64URL Encoding**:
```
eyJhbGciOiJIUzUxMiIsInR5cCI6IkpXVCJ9
```

---

### 3.2 Payload

Contains authentication credentials and request metadata.

**Required Fields**:
- `access_key` (string): API Access Key
- `nonce` (string): Unique identifier per request (UUID v4 recommended)

**Conditional Fields**:
- `query_hash` (string): SHA-512 hash of query parameters or request body
- `query_hash_alg` (string): Hash algorithm identifier (must be `"SHA512"`)

**Field Details**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `access_key` | string | Yes | Your API Access Key from Upbit account |
| `nonce` | string | Yes | Unique UUID for this request (e.g., UUID v4) |
| `query_hash` | string | Conditional | SHA-512 hash of request parameters (if parameters/body exist) |
| `query_hash_alg` | string | With query_hash | Must be `"SHA512"` when `query_hash` is present |

**Example Payload (GET request without parameters)**:
```json
{
  "access_key": "abcdef1234567890abcdef1234567890",
  "nonce": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
}
```

**Example Payload (POST request with body)**:
```json
{
  "access_key": "abcdef1234567890abcdef1234567890",
  "nonce": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "query_hash": "3f2a8b9c7d6e5f4a3b2c1d0e9f8a7b6c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f0",
  "query_hash_alg": "SHA512"
}
```

---

### 3.3 Signature

HMAC-SHA512 hash of the encoded header and payload, using the Secret Key.

**Calculation**:
```
signature = HMAC-SHA512(
  key: secret_key,
  message: "{base64url_header}.{base64url_payload}"
)
```

**Then Base64URL encode the signature**:
```
base64url_signature = base64url_encode(signature)
```

---

## 4. Query Hash Calculation

The `query_hash` is required when the request includes query parameters (GET/DELETE) or a request body (POST).

### 4.1 For GET/DELETE Requests (Query Parameters)

**Step 1**: Construct the query string from parameters (URL-encoded format)
- **Important**: Use the **exact order** of parameters as they appear in the URL
- Do **NOT** re-sort or reorder parameters
- URL-encode parameter values (except array brackets `[]`)

**Example**:
```
market=SGD-BTC&state=wait&page=1
```

**Step 2**: Calculate SHA-512 hash of the query string
```python
import hashlib

query_string = "market=SGD-BTC&state=wait&page=1"
query_hash = hashlib.sha512(query_string.encode('utf-8')).hexdigest()
```

**Step 3**: Include `query_hash` and `query_hash_alg` in JWT payload

---

### 4.2 For POST Requests (JSON Body)

**Step 1**: Convert JSON body to query string format
- Extract all key-value pairs from JSON
- Format as `key=value&key=value`
- Maintain order (preferably alphabetical for consistency)

**Example JSON Body**:
```json
{
  "market": "SGD-BTC",
  "side": "bid",
  "volume": "0.1",
  "price": "67000",
  "ord_type": "limit"
}
```

**Converted to Query String**:
```
market=SGD-BTC&ord_type=limit&price=67000&side=bid&volume=0.1
```

**Step 2**: Calculate SHA-512 hash
```python
import hashlib

query_string = "market=SGD-BTC&ord_type=limit&price=67000&side=bid&volume=0.1"
query_hash = hashlib.sha512(query_string.encode('utf-8')).hexdigest()
```

**Step 3**: Include in JWT payload

---

### 4.3 Query Hash Algorithm

**Algorithm**: SHA-512 (SHA-2 family, 512-bit hash)

**Properties**:
- Produces 128-character hexadecimal string
- Cryptographically secure
- One-way function (cannot reverse)

**Encoding**: Hexadecimal lowercase

**Example**:
```
Input: "market=SGD-BTC&side=bid&volume=0.1&price=67000&ord_type=limit"
Output: "3f2a8b9c7d6e5f4a3b2c1d0e9f8a7b6c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f0a9b8c7d6e5f4a3b2c1d0e9f8a7b6c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f0"
```

---

## 5. Token Generation Process

### 5.1 Complete Flow

**Step 1**: Generate nonce (UUID v4)
```python
import uuid

nonce = str(uuid.uuid4())  # e.g., "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
```

**Step 2**: Construct payload
```python
payload = {
    "access_key": access_key,
    "nonce": nonce
}
```

**Step 3**: If request has parameters/body, calculate query_hash
```python
import hashlib

# For GET/DELETE with query parameters
query_string = "market=SGD-BTC&state=wait"
query_hash = hashlib.sha512(query_string.encode('utf-8')).hexdigest()

# For POST with JSON body
import json
from urllib.parse import urlencode

body = {"market": "SGD-BTC", "side": "bid", "volume": "0.1"}
query_string = urlencode(sorted(body.items()))  # Sort for consistency
query_hash = hashlib.sha512(query_string.encode('utf-8')).hexdigest()

# Add to payload
payload["query_hash"] = query_hash
payload["query_hash_alg"] = "SHA512"
```

**Step 4**: Generate JWT token
```python
import jwt

token = jwt.encode(payload, secret_key, algorithm="HS512")
```

**Step 5**: Add to request header
```python
headers = {
    "Authorization": f"Bearer {token}",
    "Content-Type": "application/json; charset=utf-8"
}
```

---

### 5.2 Python Example (Complete)

```python
import hashlib
import uuid
import jwt
import requests
from urllib.parse import urlencode

# Credentials
access_key = "your_access_key"
secret_key = "your_secret_key"
base_url = "https://sg-api.upbit.com"

def create_jwt_token(access_key, secret_key, query_string=""):
    """
    Generate JWT token for Upbit API authentication.

    Args:
        access_key: API Access Key
        secret_key: API Secret Key
        query_string: URL-encoded query string (for GET/DELETE) or
                     converted JSON body (for POST)

    Returns:
        JWT token string
    """
    payload = {
        "access_key": access_key,
        "nonce": str(uuid.uuid4())
    }

    if query_string:
        query_hash = hashlib.sha512(query_string.encode('utf-8')).hexdigest()
        payload["query_hash"] = query_hash
        payload["query_hash_alg"] = "SHA512"

    return jwt.encode(payload, secret_key, algorithm="HS512")

# Example 1: GET request without parameters (e.g., /v1/balances)
token = create_jwt_token(access_key, secret_key)
headers = {
    "Authorization": f"Bearer {token}",
    "Content-Type": "application/json; charset=utf-8"
}
response = requests.get(f"{base_url}/v1/balances", headers=headers)

# Example 2: GET request with parameters (e.g., /v1/orders?market=SGD-BTC&state=wait)
query_params = {"market": "SGD-BTC", "state": "wait"}
query_string = urlencode(sorted(query_params.items()))
token = create_jwt_token(access_key, secret_key, query_string)
headers = {
    "Authorization": f"Bearer {token}",
    "Content-Type": "application/json; charset=utf-8"
}
response = requests.get(
    f"{base_url}/v1/orders",
    params=query_params,
    headers=headers
)

# Example 3: POST request with JSON body (e.g., create order)
order_data = {
    "market": "SGD-BTC",
    "side": "bid",
    "volume": "0.1",
    "price": "67000",
    "ord_type": "limit"
}
query_string = urlencode(sorted(order_data.items()))
token = create_jwt_token(access_key, secret_key, query_string)
headers = {
    "Authorization": f"Bearer {token}",
    "Content-Type": "application/json; charset=utf-8"
}
response = requests.post(
    f"{base_url}/v1/orders",
    json=order_data,
    headers=headers
)
```

---

### 5.3 Rust Example (Pseudocode)

```rust
use hmac::{Hmac, Mac};
use sha2::Sha512;
use uuid::Uuid;
use serde_json::json;
use base64::{Engine as _, engine::general_purpose};

type HmacSha512 = Hmac<Sha512>;

fn create_jwt_token(
    access_key: &str,
    secret_key: &str,
    query_string: Option<&str>,
) -> String {
    // Generate nonce
    let nonce = Uuid::new_v4().to_string();

    // Construct payload
    let mut payload = json!({
        "access_key": access_key,
        "nonce": nonce
    });

    // Add query_hash if query string exists
    if let Some(qs) = query_string {
        let query_hash = sha512_hex(qs.as_bytes());
        payload["query_hash"] = json!(query_hash);
        payload["query_hash_alg"] = json!("SHA512");
    }

    // JWT header
    let header = json!({
        "alg": "HS512",
        "typ": "JWT"
    });

    // Base64URL encode header and payload
    let header_b64 = base64url_encode(&header.to_string());
    let payload_b64 = base64url_encode(&payload.to_string());

    // Create signature
    let message = format!("{}.{}", header_b64, payload_b64);
    let signature = hmac_sha512(secret_key.as_bytes(), message.as_bytes());
    let signature_b64 = base64url_encode(&signature);

    // Combine into JWT
    format!("{}.{}.{}", header_b64, payload_b64, signature_b64)
}

fn sha512_hex(data: &[u8]) -> String {
    use sha2::Digest;
    let mut hasher = Sha512::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn hmac_sha512(key: &[u8], message: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha512::new_from_slice(key).expect("HMAC init");
    mac.update(message);
    mac.finalize().into_bytes().to_vec()
}

fn base64url_encode(data: &[u8]) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(data)
}
```

---

## 6. Nonce Requirements

### 6.1 Format

**Type**: String
**Recommended**: UUID version 4

**Example**:
```
a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

---

### 6.2 Uniqueness

**Critical**: "A new value must be provided for every request"

- Each API request **MUST** have a unique nonce
- Reusing nonces can cause authentication failures
- Use UUID v4 for guaranteed uniqueness

---

### 6.3 Generation

**Python**:
```python
import uuid
nonce = str(uuid.uuid4())
```

**Rust**:
```rust
use uuid::Uuid;
let nonce = Uuid::new_v4().to_string();
```

**JavaScript**:
```javascript
import { v4 as uuidv4 } from 'uuid';
const nonce = uuidv4();
```

---

## 7. Differences Between Public and Private Endpoints

### 7.1 Public Endpoints (Quotation API)

**Authentication**: None required
**Headers**: Standard HTTP headers
**Endpoints**:
- `/v1/trading-pairs`
- `/v1/candles/*`
- `/v1/tickers`
- `/v1/orderbooks`
- `/v1/trades/recent`

**Example**:
```python
response = requests.get("https://sg-api.upbit.com/v1/trading-pairs")
```

---

### 7.2 Private Endpoints (Exchange API)

**Authentication**: JWT Bearer token required
**Headers**:
- `Authorization: Bearer {JWT_TOKEN}`
- `Content-Type: application/json; charset=utf-8`

**Endpoints**:
- `/v1/orders`
- `/v1/balances`
- `/v1/deposits`
- `/v1/withdrawals`

**Permissions**:
- **View Account**: Read balances, orders, deposits, withdrawals
- **Make Orders**: Create and cancel orders
- **Make Deposits/Withdrawals**: Manage deposits and withdrawals

**Example**:
```python
token = create_jwt_token(access_key, secret_key)
headers = {
    "Authorization": f"Bearer {token}",
    "Content-Type": "application/json; charset=utf-8"
}
response = requests.get("https://sg-api.upbit.com/v1/balances", headers=headers)
```

---

## 8. Secret Key Handling

### 8.1 No Base64 Decoding Required

**Important**: The Secret Key is used **as-is** for signing, without Base64 decoding.

**Incorrect**:
```python
import base64
decoded_secret = base64.b64decode(secret_key)  # DON'T DO THIS
```

**Correct**:
```python
# Use secret_key directly as string
token = jwt.encode(payload, secret_key, algorithm="HS512")
```

---

### 8.2 Security Best Practices

1. **Never hardcode credentials** in source code
2. **Use environment variables** or secure configuration files
3. **Store secrets encrypted** at rest
4. **Rotate API keys** periodically
5. **Use IP whitelisting** when possible
6. **Monitor API usage** for suspicious activity
7. **Revoke compromised keys** immediately

---

## 9. Error Handling

### 9.1 Authentication Errors

**HTTP 401 Unauthorized**:
- Invalid JWT token
- Expired token
- Invalid Access Key or Secret Key
- Incorrect query_hash

**HTTP 403 Forbidden**:
- Insufficient permissions
- API key doesn't have required permission group

**Example Error Response**:
```json
{
  "error": {
    "name": "invalid_signature",
    "message": "JWT signature does not match"
  }
}
```

---

### 9.2 Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| Invalid signature | Query hash mismatch | Verify query string construction |
| Token expired | Token reuse or time skew | Generate new token for each request |
| Nonce error | Reused nonce | Ensure unique UUID for each request |
| Permission denied | Missing API permission | Check API key permissions in account |
| Query hash algorithm error | Wrong algorithm specified | Use "SHA512" for `query_hash_alg` |

---

## 10. Parameter Order Consistency

### 10.1 Critical Rule

**For GET/DELETE requests**: The query string used for `query_hash` calculation **MUST** match the exact parameter order in the actual URL.

**For POST requests**: Convert JSON to query string format. Recommended to sort alphabetically for consistency.

---

### 10.2 Example

**URL**:
```
https://sg-api.upbit.com/v1/orders?market=SGD-BTC&state=wait&page=1
```

**Query String for Hash**:
```
market=SGD-BTC&state=wait&page=1
```

**NOT**:
```
page=1&market=SGD-BTC&state=wait  # Different order - will fail!
```

---

## 11. JWT Library Recommendations

### 11.1 Python

**Library**: `PyJWT`

```bash
pip install PyJWT
```

**Usage**:
```python
import jwt
token = jwt.encode(payload, secret_key, algorithm="HS512")
```

---

### 11.2 Rust

**Library**: `jsonwebtoken`

```toml
[dependencies]
jsonwebtoken = "9"
```

**Usage**:
```rust
use jsonwebtoken::{encode, Header, EncodingKey, Algorithm};

let header = Header::new(Algorithm::HS512);
let token = encode(&header, &payload, &EncodingKey::from_secret(secret_key.as_bytes()))?;
```

---

### 11.3 Node.js

**Library**: `jsonwebtoken`

```bash
npm install jsonwebtoken
```

**Usage**:
```javascript
const jwt = require('jsonwebtoken');
const token = jwt.sign(payload, secretKey, { algorithm: 'HS512' });
```

---

## 12. Summary

### 12.1 Key Takeaways

1. **JWT Authentication**: Upbit uses JWT tokens instead of HMAC signatures
2. **HS512 Algorithm**: HMAC with SHA-512 is recommended
3. **Unique Nonce**: Each request requires a unique UUID v4 nonce
4. **Query Hash**: Required for requests with parameters or body (SHA-512)
5. **No Decoding**: Secret Key used as-is, not Base64-decoded
6. **Bearer Token**: JWT sent in `Authorization: Bearer {token}` header
7. **Parameter Order**: Must match exactly for query hash calculation
8. **Three Permissions**: View Account, Make Orders, Make Deposits/Withdrawals

---

### 12.2 Authentication Checklist

- [ ] Generate unique UUID v4 nonce for each request
- [ ] Construct payload with `access_key` and `nonce`
- [ ] Calculate `query_hash` if request has parameters/body
- [ ] Use SHA-512 for query hash
- [ ] Specify `query_hash_alg: "SHA512"` when query_hash present
- [ ] Encode JWT with HS512 algorithm
- [ ] Use Secret Key as-is (no Base64 decoding)
- [ ] Add `Authorization: Bearer {token}` header
- [ ] Add `Content-Type: application/json; charset=utf-8` header
- [ ] Ensure parameter order consistency

---

## Sources

- [Upbit Open API - Authentication Guide](https://global-docs.upbit.com/reference/auth)
- [Upbit Open API - REST API Guide](https://global-docs.upbit.com/reference/rest-api-guide)
- [JWT.io - JSON Web Token Introduction](https://www.jwt.io/introduction)
- [CCXT - Upbit Implementation](https://github.com/ccxt/ccxt/blob/master/python/ccxt/upbit.py)
- [Upbit GitHub Client Repository](https://github.com/upbit-exchange/client)

---

**Document Version**: 1.0
**Last Updated**: 2026-01-20
**Research Completed By**: Claude Code Research Agent
