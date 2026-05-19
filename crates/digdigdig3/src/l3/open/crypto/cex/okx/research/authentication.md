# OKX API v5 Authentication

## Overview

All private OKX API requests require authentication using four mandatory headers and HMAC SHA256 signature generation.

---

## Required Headers

All private REST API requests must include these four headers:

| Header | Description | Example |
|--------|-------------|---------|
| `OK-ACCESS-KEY` | API key string | `"37c541a1-XXXX-XXXX-XXXX-10840aXXXXX"` |
| `OK-ACCESS-SIGN` | Base64-encoded signature | `"VMrVeqsGTDI2vqAzIPEW0aQ...=="` |
| `OK-ACCESS-TIMESTAMP` | UTC timestamp (ISO 8601) | `"2020-12-08T09:08:57.715Z"` |
| `OK-ACCESS-PASSPHRASE` | Passphrase from API key creation | `"MyPassphrase123"` |

**Additional Header (Optional):**
- `x-simulated-trading: 1` - Enable demo trading mode

---

## Signature Algorithm

### Step 1: Create Pre-Hash String

The pre-hash string is constructed by concatenating four components **without separators**:

```
prehash = timestamp + method + requestPath + body
```

**Components:**

1. **timestamp**: Must match `OK-ACCESS-TIMESTAMP` header exactly
   - Format: ISO 8601 with milliseconds (e.g., `2020-12-08T09:08:57.715Z`)
   - Must be UTC timezone

2. **method**: HTTP method in **UPPERCASE**
   - Examples: `GET`, `POST`, `DELETE`

3. **requestPath**: The endpoint path including query parameters
   - For GET: Include query string (e.g., `/api/v5/account/balance?ccy=BTC`)
   - For POST: Path only, no query string (e.g., `/api/v5/trade/order`)

4. **body**: JSON request body as string
   - For GET: Empty string `""`
   - For POST/DELETE: Stringified JSON (e.g., `'{"instId":"BTC-USDT","tdMode":"cash"}'`)
   - Must be valid JSON with `Content-Type: application/json`

### Step 2: Sign with HMAC SHA256

Sign the pre-hash string using your **SecretKey** with HMAC SHA256 algorithm, then encode the result in **Base64**.

**Formula:**
```
signature = Base64(HMAC-SHA256(prehash, SecretKey))
```

**JavaScript Example:**
```javascript
const sign = CryptoJS.enc.Base64.stringify(
  CryptoJS.HmacSHA256(
    timestamp + 'GET' + '/api/v5/account/balance?ccy=BTC',
    SecretKey
  )
);
```

**Python Example:**
```python
import hmac
import hashlib
import base64

prehash = timestamp + method + request_path + body
signature = base64.b64encode(
    hmac.new(
        secret_key.encode('utf-8'),
        prehash.encode('utf-8'),
        hashlib.sha256
    ).digest()
).decode('utf-8')
```

**Rust Example:**
```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::Engine;

type HmacSha256 = Hmac<Sha256>;

fn sign(timestamp: &str, method: &str, path: &str, body: &str, secret: &str) -> String {
    let prehash = format!("{}{}{}{}", timestamp, method, path, body);

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(prehash.as_bytes());

    let result = mac.finalize();
    base64::engine::general_purpose::STANDARD.encode(result.into_bytes())
}
```

---

## Pre-Hash String Examples

### Example 1: GET Request with Query Parameters

**Request:**
- Method: `GET`
- Path: `/api/v5/account/balance?ccy=BTC`
- Timestamp: `2020-12-08T09:08:57.715Z`
- Body: (none)

**Pre-hash String:**
```
2020-12-08T09:08:57.715ZGET/api/v5/account/balance?ccy=BTC
```

### Example 2: POST Request with Body

**Request:**
- Method: `POST`
- Path: `/api/v5/trade/order`
- Timestamp: `2020-12-08T09:08:57.715Z`
- Body: `{"instId":"BTC-USDT","tdMode":"cash","side":"buy","ordType":"limit","px":"20000","sz":"0.01"}`

**Pre-hash String:**
```
2020-12-08T09:08:57.715ZPOST/api/v5/trade/order{"instId":"BTC-USDT","tdMode":"cash","side":"buy","ordType":"limit","px":"20000","sz":"0.01"}
```

### Example 3: GET Request without Query Parameters

**Request:**
- Method: `GET`
- Path: `/api/v5/account/config`
- Timestamp: `2020-12-08T09:08:57.715Z`
- Body: (none)

**Pre-hash String:**
```
2020-12-08T09:08:57.715ZGET/api/v5/account/config
```

---

## Complete Request Example

### GET Balance Request

**Headers:**
```
OK-ACCESS-KEY: 37c541a1-XXXX-XXXX-XXXX-10840aXXXXX
OK-ACCESS-SIGN: VMrVeqsGTDI2vqAzIPEW0aQ...==
OK-ACCESS-TIMESTAMP: 2020-12-08T09:08:57.715Z
OK-ACCESS-PASSPHRASE: MyPassphrase123
Content-Type: application/json
```

**URL:**
```
GET https://www.okx.com/api/v5/account/balance?ccy=BTC
```

### POST Order Request

**Headers:**
```
OK-ACCESS-KEY: 37c541a1-XXXX-XXXX-XXXX-10840aXXXXX
OK-ACCESS-SIGN: qL8DVke3D5M8mKW1P7oBEF...==
OK-ACCESS-TIMESTAMP: 2020-12-08T09:08:57.715Z
OK-ACCESS-PASSPHRASE: MyPassphrase123
Content-Type: application/json
```

**URL:**
```
POST https://www.okx.com/api/v5/trade/order
```

**Body:**
```json
{
  "instId": "BTC-USDT",
  "tdMode": "cash",
  "side": "buy",
  "ordType": "limit",
  "px": "20000",
  "sz": "0.01"
}
```

---

## WebSocket Authentication

WebSocket private channels require login via a special message after connection.

### WebSocket Login Message Format

```json
{
  "op": "login",
  "args": [
    {
      "apiKey": "37c541a1-XXXX-XXXX-XXXX-10840aXXXXX",
      "passphrase": "MyPassphrase123",
      "timestamp": "2020-12-08T09:08:57.715Z",
      "sign": "VMrVeqsGTDI2vqAzIPEW0aQ...=="
    }
  ]
}
```

### WebSocket Signature

For WebSocket login, the pre-hash string is:

```
prehash = timestamp + "GET" + "/users/self/verify"
```

Then sign with HMAC SHA256 and encode in Base64 (same as REST).

---

## Important Notes

### 1. Timestamp Validation
- Request expires **30 seconds** after the timestamp
- Server uses its own timestamp for validation
- Query server time first: `GET /api/v5/public/time` to avoid sync issues

### 2. Content-Type
- All POST/PUT/DELETE requests must use `Content-Type: application/json`
- Body must be valid JSON (not form-encoded)

### 3. API Key Permissions
When creating an API key, you can set permissions:
- **Read**: View account/order data
- **Trade**: Place/cancel orders
- **Withdraw**: Transfer funds (use with caution)

### 4. IP Whitelist
- Recommended for security
- Can restrict API key to specific IP addresses
- Configure in OKX account settings

### 5. API Key Expiration
- Unused API keys expire after **14 days** of inactivity
- Refresh by making any authenticated request

### 6. Demo Trading
- Use same authentication mechanism
- Add header: `x-simulated-trading: 1`
- Demo and production keys are separate

### 7. Request Body for GET
- GET requests **never** include a body in the signature
- Query parameters are part of the `requestPath` component
- Empty string `""` for body component in pre-hash

### 8. Signature Encoding
- HMAC SHA256 result must be Base64-encoded
- **Not** hex-encoded
- Use standard Base64 alphabet (not URL-safe variant)

---

## Error Codes

| Code | Message | Description |
|------|---------|-------------|
| 50101 | API frozen | API key permissions insufficient |
| 50102 | Timestamp request expired | Request > 30 seconds old |
| 50103 | Request header "OK-ACCESS-KEY" cannot be empty | Missing API key header |
| 50104 | Request header "OK-ACCESS-PASSPHRASE" cannot be empty | Missing passphrase header |
| 50105 | Request header "OK-ACCESS-TIMESTAMP" cannot be empty | Missing timestamp header |
| 50106 | Request header "OK-ACCESS-SIGN" cannot be empty | Missing signature header |
| 50107 | Invalid OK-ACCESS-TIMESTAMP | Timestamp format incorrect |
| 50111 | Invalid sign | Signature verification failed |
| 50113 | Invalid IP | IP not whitelisted |

---

## Security Best Practices

1. **Never expose your SecretKey** - Store securely, never commit to git
2. **Use IP whitelisting** - Restrict API key to known IPs
3. **Minimal permissions** - Only enable required permissions
4. **Rotate keys regularly** - Regenerate API keys periodically
5. **Monitor API activity** - Check for unauthorized usage
6. **Use HTTPS only** - Never send credentials over HTTP
7. **Validate server certificates** - Prevent MITM attacks

---

## Implementation Checklist

- [ ] Generate ISO 8601 timestamp with milliseconds
- [ ] Construct pre-hash string: `timestamp + method + path + body`
- [ ] Sign with HMAC SHA256 using SecretKey
- [ ] Encode signature in Base64
- [ ] Set all four required headers
- [ ] Handle timestamp expiration (30s window)
- [ ] Implement error handling for auth failures (50xxx codes)
- [ ] Store SecretKey securely (environment variables, key vault)
- [ ] Test with both GET and POST requests
- [ ] Verify signature with different body content
