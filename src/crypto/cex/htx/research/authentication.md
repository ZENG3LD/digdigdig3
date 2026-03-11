# HTX API Authentication

Complete authentication and signature documentation for HTX (formerly Huobi) exchange.

## Overview

HTX uses **HMAC SHA256** signature-based authentication for all private API endpoints. Each request must include:

1. API Access Key (public identifier)
2. Signature Method (always HmacSHA256)
3. Signature Version (always 2)
4. Timestamp (UTC, valid within 5 minutes)
5. Signature (computed HMAC)

## API Key Creation

### Creating API Keys

1. Maximum 20 API keys per user
2. Each key can have up to 4 bound IP addresses (optional but recommended)
3. IP bindings expire after 90 days if not renewed
4. Sub-users: Max 200 per parent, 5 API keys per sub-user

### API Key Permissions

Three permission levels can be assigned:
- **Read**: Query data only
- **Trade**: Place/cancel orders, transfers
- **Withdraw**: Withdrawal operations

**Note:** API keys are shared across all instruments (spot, futures, swap, options)

## Signature Process

### Step 1: Construct Pre-Sign String

The signature is computed over a canonical request string in the format:

```
<HTTP_METHOD>\n
<HOST>\n
<PATH>\n
<QUERY_STRING>
```

**Components:**
- `HTTP_METHOD`: GET, POST, etc. (uppercase)
- `HOST`: `api.huobi.pro` or `api-aws.huobi.pro`
- `PATH`: `/v1/order/orders/place`
- `QUERY_STRING`: URL-encoded parameters in ASCII order

### Step 2: Required Parameters

All authenticated requests must include these parameters in the query string (even for POST requests):

```
AccessKeyId=<your-access-key>
SignatureMethod=HmacSHA256
SignatureVersion=2
Timestamp=<UTC-timestamp>
```

**Timestamp Format:**
- UTC format: `YYYY-MM-DDThh:mm:ss`
- URL encoded: `2023-01-20T12%3A34%3A56`
- Valid window: ±5 minutes from server time
- Use `GET /v1/common/timestamp` to get server time

### Step 3: Sort Parameters

Sort all query parameters (including auth params) in **ASCII ascending order**.

Example before sorting:
```
symbol=btcusdt&AccessKeyId=xxx&SignatureMethod=HmacSHA256&Timestamp=2023-01-20T12:34:56
```

Example after sorting:
```
AccessKeyId=xxx&SignatureMethod=HmacSHA256&Timestamp=2023-01-20T12:34:56&symbol=btcusdt
```

### Step 4: Compute HMAC SHA256

1. Concatenate the components with newlines
2. Compute HMAC SHA256 using your Secret Key
3. Base64 encode the result
4. URL encode the Base64 string

```rust
// Pseudo-code
let pre_sign_string = format!(
    "{}\n{}\n{}\n{}",
    method,      // "GET" or "POST"
    host,        // "api.huobi.pro"
    path,        // "/v1/order/orders/place"
    query_string // "AccessKeyId=xxx&SignatureMethod=..."
);

let signature = hmac_sha256(secret_key, pre_sign_string);
let signature_b64 = base64_encode(signature);
let signature_encoded = url_encode(signature_b64);
```

### Step 5: Append Signature

Add the computed signature to the query string:

```
https://api.huobi.pro/v1/order/orders/place?AccessKeyId=xxx&SignatureMethod=HmacSHA256&SignatureVersion=2&Timestamp=2023-01-20T12%3A34%3A56&symbol=btcusdt&Signature=<computed-signature>
```

## Request Examples

### Example 1: GET Request

**Endpoint:** `GET /v1/order/openOrders`

**Step 1:** Construct query parameters
```
account-id=123456
symbol=btcusdt
AccessKeyId=abcd1234-ef56-7890-abcd-ef1234567890
SignatureMethod=HmacSHA256
SignatureVersion=2
Timestamp=2023-01-20T12:34:56
```

**Step 2:** Sort parameters (ASCII order)
```
AccessKeyId=abcd1234-ef56-7890-abcd-ef1234567890
SignatureMethod=HmacSHA256
SignatureVersion=2
Timestamp=2023-01-20T12:34:56
account-id=123456
symbol=btcusdt
```

**Step 3:** URL encode and join
```
AccessKeyId=abcd1234-ef56-7890-abcd-ef1234567890&SignatureMethod=HmacSHA256&SignatureVersion=2&Timestamp=2023-01-20T12%3A34%3A56&account-id=123456&symbol=btcusdt
```

**Step 4:** Build pre-sign string
```
GET\n
api.huobi.pro\n
/v1/order/openOrders\n
AccessKeyId=abcd1234-ef56-7890-abcd-ef1234567890&SignatureMethod=HmacSHA256&SignatureVersion=2&Timestamp=2023-01-20T12%3A34%3A56&account-id=123456&symbol=btcusdt
```

**Step 5:** Compute signature
```rust
let signature = hmac_sha256("your-secret-key", pre_sign_string);
let signature_b64 = base64_encode(signature);
let signature_encoded = url_encode(signature_b64);
```

**Step 6:** Final URL
```
https://api.huobi.pro/v1/order/openOrders?AccessKeyId=abcd1234-ef56-7890-abcd-ef1234567890&SignatureMethod=HmacSHA256&SignatureVersion=2&Timestamp=2023-01-20T12%3A34%3A56&account-id=123456&symbol=btcusdt&Signature=<signature>
```

### Example 2: POST Request

**Endpoint:** `POST /v1/order/orders/place`

**Request Body (JSON):**
```json
{
  "account-id": "123456",
  "symbol": "btcusdt",
  "type": "buy-limit",
  "amount": "0.1",
  "price": "50000.00"
}
```

**Step 1:** Construct auth parameters (query string only)
```
AccessKeyId=abcd1234-ef56-7890-abcd-ef1234567890
SignatureMethod=HmacSHA256
SignatureVersion=2
Timestamp=2023-01-20T12:34:56
```

**Step 2:** Pre-sign string (no body parameters)
```
POST\n
api.huobi.pro\n
/v1/order/orders/place\n
AccessKeyId=abcd1234-ef56-7890-abcd-ef1234567890&SignatureMethod=HmacSHA256&SignatureVersion=2&Timestamp=2023-01-20T12%3A34%3A56
```

**Step 3:** Compute signature (same process)

**Step 4:** Make request
```
POST https://api.huobi.pro/v1/order/orders/place?AccessKeyId=xxx&SignatureMethod=HmacSHA256&SignatureVersion=2&Timestamp=xxx&Signature=xxx

Headers:
  Content-Type: application/json

Body:
{
  "account-id": "123456",
  "symbol": "btcusdt",
  "type": "buy-limit",
  "amount": "0.1",
  "price": "50000.00"
}
```

**IMPORTANT:** For POST requests:
- Auth parameters go in query string
- Business parameters go in JSON body
- Only auth parameters are included in signature computation
- Content-Type MUST be `application/json`

## WebSocket Authentication

### WebSocket V2 Authentication

For private WebSocket endpoints (`wss://api.huobi.pro/ws/v2`), authentication is done during connection:

**Step 1:** Connect to WebSocket

**Step 2:** Send authentication message
```json
{
  "action": "req",
  "ch": "auth",
  "params": {
    "authType": "api",
    "accessKey": "abcd1234-ef56-7890-abcd-ef1234567890",
    "signatureMethod": "HmacSHA256",
    "signatureVersion": "2.1",
    "timestamp": "2023-01-20T12:34:56",
    "signature": "<computed-signature>"
  }
}
```

**Signature Computation for WebSocket:**

Pre-sign string:
```
GET\n
api.huobi.pro\n
/ws/v2\n
accessKey=xxx&signatureMethod=HmacSHA256&signatureVersion=2.1&timestamp=2023-01-20T12:34:56
```

**Step 3:** Receive authentication response
```json
{
  "action": "req",
  "code": 200,
  "ch": "auth",
  "data": {}
}
```

Success: `code: 200`
Failure: `code: 2002` (invalid signature), `code: 2003` (timestamp error)

## Common Authentication Errors

### Error Codes

| Error Code | Message | Cause | Solution |
|------------|---------|-------|----------|
| `api-signature-not-valid` | Signature verification failed | Invalid signature | Verify signature computation |
| `api-signature-check-failed` | Signature check failed | Incorrect secret key | Use correct secret key |
| `invalid-parameter` | Invalid parameter | Missing required params | Check all required parameters |
| `login-required` | Login required | Missing auth parameters | Include AccessKeyId, Signature |
| `invalid-timestamp` | Invalid timestamp | Timestamp out of window | Use server time ±5 minutes |
| `api-key-invalid` | Invalid API key | API key doesn't exist | Create valid API key |
| `gateway-internal-error` | Internal error | Server error | Retry request |

### Troubleshooting

**Issue: Signature verification failed**
- Ensure parameters are sorted in ASCII order
- Verify URL encoding is correct
- Check timestamp format (YYYY-MM-DDThh:mm:ss)
- Ensure newlines (`\n`) in pre-sign string
- Use correct host (api.huobi.pro)
- Verify HTTP method is uppercase (GET, POST)

**Issue: Invalid timestamp**
- Use `GET /v1/common/timestamp` to get server time
- Ensure system clock is synchronized
- Timestamp must be within ±5 minutes of server time
- URL encode the timestamp properly (`:` becomes `%3A`)

**Issue: GET request with body returns 403**
- GET requests must NOT have a request body
- All parameters must be in query string
- Content-Length must be 0 for GET requests

## Security Best Practices

1. **Never expose Secret Key**: Store securely, never commit to version control
2. **Use IP whitelisting**: Bind API keys to specific IP addresses
3. **Minimum permissions**: Grant only necessary permissions (Read/Trade/Withdraw)
4. **Rotate keys regularly**: Create new keys periodically
5. **Monitor API usage**: Check for unauthorized access
6. **Use HTTPS only**: Never use unencrypted HTTP
7. **Validate responses**: Check response status and error codes
8. **Sub-user isolation**: Use sub-users for API isolation

## Rate Limit Headers

Monitor these response headers for rate limit status:

```
X-HB-RateLimit-Requests-Remain: 95
X-HB-RateLimit-Requests-Expire: 1234567890
```

- `X-HB-RateLimit-Requests-Remain`: Remaining requests in current window
- `X-HB-RateLimit-Requests-Expire`: Window expiration timestamp (ms)

Rate limits are applied **per UID** across all API keys, not per API key.

## Implementation Notes

### URL Encoding Rules

- Space: `%20` (not `+`)
- Colon: `%3A`
- Equals: `%3D` (in values, not parameter separators)
- Ampersand: `%26` (in values, not parameter separators)
- Forward slash: `%2F`

### Parameter Ordering

Use **byte-wise ASCII sorting**:
```
Numbers (0-9) < Uppercase (A-Z) < Lowercase (a-z)
```

Example order:
```
AccessKeyId
SignatureMethod
SignatureVersion
Timestamp
account-id
amount
price
symbol
type
```

### Signature Algorithm

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{Engine as _, engine::general_purpose};

type HmacSha256 = Hmac<Sha256>;

fn sign(secret: &str, message: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(message.as_bytes());
    let result = mac.finalize();
    let bytes = result.into_bytes();
    general_purpose::STANDARD.encode(bytes)
}
```

## Reference Implementation

### Complete GET Request Example

```rust
use chrono::Utc;
use std::collections::BTreeMap;

fn build_signed_request(
    method: &str,
    host: &str,
    path: &str,
    access_key: &str,
    secret_key: &str,
    mut params: BTreeMap<String, String>,
) -> String {
    // Add auth parameters
    params.insert("AccessKeyId".to_string(), access_key.to_string());
    params.insert("SignatureMethod".to_string(), "HmacSHA256".to_string());
    params.insert("SignatureVersion".to_string(), "2".to_string());
    params.insert(
        "Timestamp".to_string(),
        Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
    );

    // Build query string (BTreeMap auto-sorts by key)
    let query_string: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, url_encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    // Build pre-sign string
    let pre_sign = format!("{}\n{}\n{}\n{}", method, host, path, query_string);

    // Compute signature
    let signature = sign(secret_key, &pre_sign);
    let signature_encoded = url_encode(&signature);

    // Build final URL
    format!(
        "https://{}{}?{}&Signature={}",
        host, path, query_string, signature_encoded
    )
}
```

## Conclusion

HTX authentication requires careful attention to:
- Proper parameter sorting (ASCII order)
- Correct URL encoding
- Accurate timestamp formatting
- HMAC SHA256 + Base64 encoding
- Signature appended to query string

Follow this documentation exactly to ensure successful authentication.
