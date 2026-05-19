# KuCoin API Authentication Research

**Research Date**: 2026-01-20
**API Documentation Version**: 2026 (Latest)

---

## 1. Required Headers for Authenticated Requests

All private REST requests to KuCoin API **MUST** include the following headers:

| Header Name | Description | Example Value |
|-------------|-------------|---------------|
| `KC-API-KEY` | The API key as a string | `"67b3c5d2a9..."` |
| `KC-API-SIGN` | The base64-encoded signature | `"Hx3ZQ8r..."` |
| `KC-API-TIMESTAMP` | Timestamp in milliseconds | `"1737379200000"` |
| `KC-API-PASSPHRASE` | Encrypted passphrase (for v2/v3) or plaintext (for v1) | `"base64_encrypted_pass"` |
| `KC-API-KEY-VERSION` | API key version (`"2"` or `"3"` recommended) | `"2"` |
| `Content-Type` | Request/response format | `"application/json"` |

---

## 2. Signature Algorithm

### 2.1 What String is Signed?

The **prehash string** format is:
```
{timestamp + method + endpoint + body}
```

### 2.2 Component Order (EXACT)

1. **Timestamp** - milliseconds (same value as `KC-API-TIMESTAMP` header)
2. **Method** - HTTP method in **UPPERCASE** (`GET`, `POST`, `DELETE`, etc.)
3. **Endpoint** - Request path (see sections 2.3-2.4 for query string handling)
4. **Body** - Request body (see section 2.5)

### 2.3 Query String Handling for GET/DELETE Requests

**CRITICAL**: For GET and DELETE requests, the endpoint **MUST include the full query string**.

**Example**:
- Full URL: `https://api.kucoin.com/api/v1/deposit-addresses?currency=BTC`
- Endpoint for signature: `/api/v1/deposit-addresses?currency=BTC`
- Prehash string: `1737379200000GET/api/v1/deposit-addresses?currency=BTC`

**URL Encoding Note**: When generating signature, use the content that has **NOT been URL-encoded**. For example:
- URL: `/api/v1/sub/api-key?apiKey=67b3&subName=test&passphrase=abc!@#11`
- Use the **original information** (not URL-encoded) for signature

### 2.4 Query String Handling for POST Requests

For POST requests:
- All query parameters go in the **JSON request body**
- The endpoint path in the signature **does NOT include query string**
- Example: `{"currency":"BTC"}` in body, endpoint is just `/api/v1/deposit-addresses`

### 2.5 Body Handling

- **GET/DELETE requests**: Body is empty string `""`
- **POST requests**: Body is the JSON string (must be identical to what's sent in request)
- **Important**: Do NOT include extra spaces in JSON strings

### 2.6 HMAC Algorithm

**Algorithm**: HMAC-SHA256

**Steps**:
1. Create prehash string: `timestamp + method + endpoint + body`
2. Use `API-Secret` as the HMAC key
3. Apply HMAC-SHA256 to the prehash string
4. Base64 encode the result
5. This is your `KC-API-SIGN` header value

### 2.7 Encoding

**Encoding**: Base64

The HMAC-SHA256 digest is **Base64 encoded** before being sent in the `KC-API-SIGN` header.

### 2.8 Python Example (Official)

```python
import time
import base64
import hmac
import hashlib
import json

api_key = "your_api_key"
api_secret = "your_api_secret"
api_passphrase = "your_passphrase"

# Example: GET request with query string
url = 'https://api.kucoin.com/api/v1/accounts?currency=BTC'
now = int(time.time() * 1000)
str_to_sign = str(now) + 'GET' + '/api/v1/accounts?currency=BTC'

signature = base64.b64encode(
    hmac.new(api_secret.encode('utf-8'),
             str_to_sign.encode('utf-8'),
             hashlib.sha256).digest())

# Example: POST request with JSON body
url = 'https://api.kucoin.com/api/v1/deposit-addresses'
now = int(time.time() * 1000)
data = {"currency":"BTC"}
data_json = json.dumps(data)
str_to_sign = str(now) + 'POST' + '/api/v1/deposit-addresses' + data_json

signature = base64.b64encode(
    hmac.new(api_secret.encode('utf-8'),
             str_to_sign.encode('utf-8'),
             hashlib.sha256).digest())
```

---

## 3. Passphrase Handling

### 3.1 API Key Version 1.0 (DEPRECATED)

- **KC-API-KEY-VERSION**: Not required or set to `"1"`
- **KC-API-PASSPHRASE**: Sent as **plaintext** (NOT encrypted)
- **Status**: Version 1.0 keys are **no longer valid** as of 2020

### 3.2 API Key Version 2.0 (RECOMMENDED)

- **KC-API-KEY-VERSION**: `"2"`
- **KC-API-PASSPHRASE**: **Encrypted** using HMAC-SHA256 + Base64
- **Encryption Algorithm**:
  1. Apply HMAC-SHA256 to passphrase using `API-Secret` as key
  2. Base64 encode the result
  3. Send encoded value in `KC-API-PASSPHRASE` header

### 3.3 API Key Version 3.0

- **KC-API-KEY-VERSION**: `"3"`
- **KC-API-PASSPHRASE**: **Encrypted** (same as v2.0)
- **Encryption Algorithm**: Same as v2.0 (HMAC-SHA256 + Base64)

### 3.4 Passphrase Encryption Example

```python
api_secret = "your_api_secret"
api_passphrase = "your_passphrase"

encrypted_passphrase = base64.b64encode(
    hmac.new(api_secret.encode('utf-8'),
             api_passphrase.encode('utf-8'),
             hashlib.sha256).digest())

# Use encrypted_passphrase in KC-API-PASSPHRASE header
```

### 3.5 Which Version to Use?

**Recommendation**: Use API Key Version **2.0** or **3.0** (both use encrypted passphrase).

Version 1.0 is deprecated and no longer supported by KuCoin.

---

## 4. Timestamp

### 4.1 Format

**Format**: Milliseconds (NOT seconds)

**Example**: `1737379200000` (milliseconds since Unix epoch)

### 4.2 Header Name

**Header**: `KC-API-TIMESTAMP`

### 4.3 Timezone Considerations

**Timezone**: UTC (Unix timestamp is timezone-agnostic)

### 4.4 Important Note

The timestamp used in the **signature prehash string** **MUST** match exactly with the value in the `KC-API-TIMESTAMP` header.

**Correct**:
```python
now = int(time.time() * 1000)
str_to_sign = str(now) + 'GET' + '/api/v1/accounts'
headers = {
    "KC-API-TIMESTAMP": str(now),  # Same timestamp
    # ...
}
```

---

## 5. Differences Between Spot and Futures

### 5.1 Authentication Headers

**Conclusion**: **NO DIFFERENCE**

Both Spot and Futures APIs use the **exact same headers**:
- `KC-API-KEY`
- `KC-API-SIGN`
- `KC-API-TIMESTAMP`
- `KC-API-PASSPHRASE`
- `KC-API-KEY-VERSION`

### 5.2 Signature Algorithm

**Conclusion**: **NO DIFFERENCE**

Both use the same signature generation process:
- Prehash string: `timestamp + method + endpoint + body`
- Algorithm: HMAC-SHA256
- Encoding: Base64

### 5.3 API Permissions

The only difference is in **API key permissions**:
- Spot trading permissions (order placement, cancellation, etc.)
- Futures trading permissions (order placement, cancellation, etc.)

These are **permission settings** on the API key itself, not differences in authentication implementation.

### 5.4 Base URLs

Different base URLs are used:
- **Spot API**: `https://api.kucoin.com`
- **Futures API**: `https://api-futures.kucoin.com`

But the authentication mechanism is identical.

---

## 6. Example from Official Documentation

### 6.1 Complete GET Request Example

```python
import time
import base64
import hmac
import hashlib

api_key = "api_key"
api_secret = "api_secret"
api_passphrase = "api_passphrase"

url = 'https://api.kucoin.com/api/v1/accounts'
now = int(time.time() * 1000)

# Signature: timestamp + method + endpoint (no query) + body (empty)
str_to_sign = str(now) + 'GET' + '/api/v1/accounts'

signature = base64.b64encode(
    hmac.new(api_secret.encode('utf-8'),
             str_to_sign.encode('utf-8'),
             hashlib.sha256).digest())

# Encrypted passphrase (for v2/v3)
passphrase = base64.b64encode(
    hmac.new(api_secret.encode('utf-8'),
             api_passphrase.encode('utf-8'),
             hashlib.sha256).digest())

headers = {
    "KC-API-SIGN": signature,
    "KC-API-TIMESTAMP": str(now),
    "KC-API-KEY": api_key,
    "KC-API-PASSPHRASE": passphrase,
    "KC-API-KEY-VERSION": "2"
}
```

### 6.2 Complete POST Request Example

```python
import time
import base64
import hmac
import hashlib
import json

api_key = "api_key"
api_secret = "api_secret"
api_passphrase = "api_passphrase"

url = 'https://api.kucoin.com/api/v1/deposit-addresses'
now = int(time.time() * 1000)

# Request body (JSON)
data = {"currency":"BTC"}
data_json = json.dumps(data)

# Signature: timestamp + method + endpoint + JSON body
str_to_sign = str(now) + 'POST' + '/api/v1/deposit-addresses' + data_json

signature = base64.b64encode(
    hmac.new(api_secret.encode('utf-8'),
             str_to_sign.encode('utf-8'),
             hashlib.sha256).digest())

passphrase = base64.b64encode(
    hmac.new(api_secret.encode('utf-8'),
             api_passphrase.encode('utf-8'),
             hashlib.sha256).digest())

headers = {
    "KC-API-SIGN": signature,
    "KC-API-TIMESTAMP": str(now),
    "KC-API-KEY": api_key,
    "KC-API-PASSPHRASE": passphrase,
    "KC-API-KEY-VERSION": "2",
    "Content-Type": "application/json"
}
```

### 6.3 GET Request with Query Parameters Example

```python
url = 'https://api-futures.kucoin.com/api/v1/position?symbol=XBTUSDM'
now = int(time.time() * 1000)

# IMPORTANT: Endpoint includes query string for signature
str_to_sign = str(now) + 'GET' + '/api/v1/position?symbol=XBTUSDM'

signature = base64.b64encode(
    hmac.new(api_secret.encode('utf-8'),
             str_to_sign.encode('utf-8'),
             hashlib.sha256).digest())

# ... rest of headers same as above
```

---

## 7. Comparison with Current auth.rs Implementation

### 7.1 Current Implementation Review

File: `v5/src/exchanges/kucoin/auth.rs`

```rust
// Current signature generation (line 59)
let sign_string = format!("{}{}{}{}", timestamp_str, method.to_uppercase(), endpoint, body);
let signature = encode_base64(&hmac_sha256(
    self.api_secret.as_bytes(),
    sign_string.as_bytes(),
));

// Current passphrase encryption (line 66)
let encrypted_passphrase = encode_base64(&hmac_sha256(
    self.api_secret.as_bytes(),
    self.passphrase.as_bytes(),
));
```

### 7.2 Discrepancies Found

**NO DISCREPANCIES** - Current implementation is **CORRECT**!

The implementation matches the official documentation exactly:

1. **Signature format**: `timestamp + method + endpoint + body` ✅
2. **Method uppercase**: `method.to_uppercase()` ✅
3. **HMAC-SHA256**: Uses `hmac_sha256()` utility ✅
4. **Base64 encoding**: Uses `encode_base64()` ✅
5. **Passphrase encryption**: HMAC-SHA256 + Base64 ✅
6. **Headers**: All required headers present ✅
7. **KC-API-KEY-VERSION**: Set to `"2"` ✅

### 7.3 Potential Considerations

**Query String Handling**: The current implementation expects the caller to pass the full endpoint path (including query string for GET/DELETE requests). This is correct, but should be documented.

**Example**:
```rust
// For GET request with query params, caller must pass full path:
auth.sign_request("GET", "/api/v1/accounts?currency=BTC", "");

// For POST request, endpoint without query params, body as JSON:
auth.sign_request("POST", "/api/v1/deposit-addresses", r#"{"currency":"BTC"}"#);
```

This is the **correct approach** and matches the official documentation.

---

## 8. Summary

### 8.1 Key Takeaways

1. **Required Headers**: 6 headers required (`KC-API-KEY`, `KC-API-SIGN`, `KC-API-TIMESTAMP`, `KC-API-PASSPHRASE`, `KC-API-KEY-VERSION`, `Content-Type`)
2. **Signature**: `HMAC-SHA256(timestamp + method + endpoint + body)` then Base64 encode
3. **Passphrase**: Must be encrypted with HMAC-SHA256 + Base64 for v2/v3 keys
4. **Timestamp**: Milliseconds (not seconds)
5. **Query Strings**: Include in endpoint for GET/DELETE, put in body for POST
6. **Spot vs Futures**: No authentication differences, same headers and signature
7. **Current Implementation**: **CORRECT** - matches official documentation exactly

### 8.2 Official Documentation Sources

- **Primary**: https://www.kucoin.com/docs-new/authentication
- **Alternative**: https://www.kucoin.com/docs/basic-info/connection-method/authentication/
- **Signing**: https://www.kucoin.com/docs/basic-info/connection-method/authentication/signing-a-message
- **Creating Request**: https://www.kucoin.com/docs/basic-info/connection-method/authentication/creating-a-request
- **API Key Upgrade**: https://www.kucoin.com/support/900006465403

---

## Sources

- [Authentication - KUCOIN API](https://www.kucoin.com/docs-new/authentication)
- [Signing a Message | KuCoin API Documentation](https://www.kucoin.com/docs/basic-info/connection-method/authentication/signing-a-message)
- [Creating a Request | KuCoin API Documentation](https://www.kucoin.com/docs/basic-info/connection-method/authentication/creating-a-request)
- [KuCoin API Key Upgrade Guideline](https://www.kucoin.com/support/900006465403)
- [GitHub - KuCoin API Demo](https://github.com/Kucoin/kucoin-api-demo)
- [GitHub - KuCoin API Documentation](https://github.com/Kucoin/kucoin-api-docs)

---

**Research completed**: 2026-01-20
**Implementation status**: Current `auth.rs` is **CORRECT** and matches official documentation
