# Bitstamp Authentication

Bitstamp API supports two authentication methods. The V2 method is recommended for all new implementations.

---

## V2 Authentication Method (Recommended)

The V2 authentication method uses HMAC-SHA256 signatures with a comprehensive string-to-sign construction.

### Required Credentials

- **API Key**: Your Bitstamp API key
- **API Secret**: Your Bitstamp API secret
- **Customer ID**: Not used in V2 method

### Authentication Headers

Every authenticated request must include the following headers:

| Header | Description |
|--------|-------------|
| `X-Auth` | `"BITSTAMP " + api_key` |
| `X-Auth-Signature` | HMAC-SHA256 signature (uppercase hex) |
| `X-Auth-Nonce` | Random UUID v4 string (36 characters, lowercase) |
| `X-Auth-Timestamp` | Request timestamp in milliseconds (UTC) |
| `X-Auth-Version` | `"v2"` |
| `Content-Type` | `"application/x-www-form-urlencoded"` (if body present) |

### String to Sign

The signature is computed over the following concatenated string:

```
BITSTAMP <api_key>
<http_method>
<host>
<path>
<query_string>
<content_type>
<nonce>
<timestamp>
<version>
<request_body>
```

**Important Notes**:
- Each component is separated by a newline (`\n`)
- `content_type` should be omitted from the string if the request body is empty
- The method name (e.g., `POST`, `GET`) must be uppercase
- `host` is the domain without protocol (e.g., `www.bitstamp.net`)
- `path` includes the full path (e.g., `/api/v2/balance/`)
- `query_string` includes the `?` if present (e.g., `?limit=100`), empty string if no query
- `request_body` is the form-encoded POST data (if applicable)

### Signature Generation

```rust
// Pseudo-code
let string_to_sign = format!(
    "BITSTAMP {}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
    api_key,
    http_method,        // e.g., "POST"
    host,               // e.g., "www.bitstamp.net"
    path,               // e.g., "/api/v2/balance/"
    query_string,       // e.g., "" or "?limit=100"
    content_type,       // e.g., "application/x-www-form-urlencoded"
    nonce,              // e.g., "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
    timestamp,          // e.g., "1643640186000"
    version,            // e.g., "v2"
    request_body        // e.g., "amount=1.0&price=1000"
);

let signature = hmac_sha256(api_secret, string_to_sign);
let signature_hex = to_uppercase_hex(signature);
```

### Example Request

**Endpoint**: `POST /api/v2/balance/`

**Headers**:
```
X-Auth: BITSTAMP myapikey123
X-Auth-Signature: A1B2C3D4E5F6...
X-Auth-Nonce: 550e8400-e29b-41d4-a716-446655440000
X-Auth-Timestamp: 1643640186000
X-Auth-Version: v2
Content-Type: application/x-www-form-urlencoded
```

**String to Sign** (empty body example):
```
BITSTAMP myapikey123
POST
www.bitstamp.net
/api/v2/balance/


550e8400-e29b-41d4-a716-446655440000
1643640186000
v2

```

**Note**: The trailing newline is included even if the body is empty.

### Example with Request Body

**Endpoint**: `POST /api/v2/buy/btcusd/`

**Body**: `amount=1.0&price=1000.00`

**String to Sign**:
```
BITSTAMP myapikey123
POST
www.bitstamp.net
/api/v2/buy/btcusd/

application/x-www-form-urlencoded
550e8400-e29b-41d4-a716-446655440000
1643640186000
v2
amount=1.0&price=1000.00
```

### Timestamp Validation

- The timestamp must be in **milliseconds** (not seconds)
- The timestamp must be within **150 seconds** of the current server time
- Requests with timestamps outside this window will be rejected

### Nonce Requirements

- Must be a unique, random string for each request
- Recommended: UUID v4 format (36 characters, lowercase with hyphens)
- Must strictly increase between requests (in practice, using UUID ensures uniqueness)
- Reusing a nonce will cause request rejection

---

## Legacy Authentication Method (Form-Based)

This method is widely used on `/api/v2` private endpoints with POST requests.

### Required Credentials

- **API Key**: Your Bitstamp API key
- **API Secret**: Your Bitstamp API secret
- **Customer ID**: Your Bitstamp customer ID

### Signature Generation

```
signature = UPPERCASE_HEX( HMAC_SHA256( api_secret, nonce + customer_id + api_key ) )
```

### Request Parameters

All authenticated requests must include these parameters in the POST body:

| Parameter | Description |
|-----------|-------------|
| `key` | Your API key |
| `signature` | HMAC-SHA256 signature (uppercase hex) |
| `nonce` | Incrementing integer (timestamp in milliseconds recommended) |

### Example

```rust
let nonce = current_timestamp_millis();  // e.g., 1643640186000
let message = format!("{}{}{}", nonce, customer_id, api_key);
let signature = hmac_sha256(api_secret, message);
let signature_hex = to_uppercase_hex(signature);

// POST body
// key=myapikey&signature=A1B2C3D4...&nonce=1643640186000&amount=1.0&price=1000
```

### Nonce in Legacy Method

- Must be an integer that increases with each request
- Recommended: Use current Unix timestamp in milliseconds
- The nonce must be greater than the previous request's nonce
- Using the same nonce twice will result in rejection

---

## Content-Type

For authenticated POST requests with a body:
- **Content-Type**: `application/x-www-form-urlencoded`

For authenticated POST requests without a body:
- **Content-Type**: Can be omitted or set to `application/x-www-form-urlencoded`

---

## Implementation Notes

### V2 Method Advantages

1. **No Customer ID Required**: Simplifies credential management
2. **More Secure**: Signs the entire request (method, path, query, body)
3. **Better Nonce Handling**: UUID-based nonces eliminate ordering issues
4. **Timestamp Protection**: 150-second window prevents replay attacks

### Recommendation

**Use V2 authentication** for all new implementations. The legacy method is only documented for compatibility with older code.

### Host and Path

- **Host**: `www.bitstamp.net` (without `https://`)
- **Path**: Full API path including `/api/v2/` prefix
- **Query String**: Include the `?` if present, otherwise empty string

### HMAC Algorithm

- **Algorithm**: HMAC-SHA256
- **Output**: Uppercase hexadecimal string
- **Key**: API secret (as UTF-8 string)
- **Message**: String to sign (as UTF-8 string)

### Error Responses

**Invalid Signature**:
```json
{
  "status": "error",
  "reason": "Invalid signature",
  "code": "API0007"
}
```

**Invalid Timestamp**:
```json
{
  "status": "error",
  "reason": "Timestamp is too far from current time",
  "code": "API0011"
}
```

**Nonce Reuse**:
```json
{
  "status": "error",
  "reason": "Nonce must be unique",
  "code": "API0012"
}
```

---

## Testing Authentication

### Test Endpoint

Use the balance endpoint to test authentication:

```
POST /api/v2/balance/
```

A successful response indicates correct authentication implementation.

### Common Issues

1. **Signature mismatch**:
   - Verify string-to-sign construction
   - Ensure newlines are correct
   - Check that signature is uppercase hex

2. **Timestamp errors**:
   - Use milliseconds, not seconds
   - Ensure system clock is synchronized
   - Check timestamp is current (not cached)

3. **Nonce errors**:
   - Use UUID v4 for V2 method
   - Ensure uniqueness across requests
   - For legacy method, use incrementing values

4. **Content-Type issues**:
   - Omit Content-Type from string-to-sign if body is empty
   - Use exact string: `application/x-www-form-urlencoded`

---

## Security Best Practices

1. **Store API credentials securely**:
   - Never commit secrets to version control
   - Use environment variables or secure key storage
   - Rotate keys periodically

2. **Use HTTPS**:
   - All API requests must use HTTPS
   - Bitstamp will reject HTTP requests

3. **Implement rate limiting**:
   - Respect Bitstamp's rate limits (see rate_limits.md)
   - Handle 429 errors gracefully

4. **Validate server certificate**:
   - Ensure TLS/SSL certificate validation is enabled
   - Prevent man-in-the-middle attacks

5. **Generate fresh nonces**:
   - Never reuse nonces
   - Use cryptographically secure random number generators

---

## Code Example (Rust)

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

fn generate_v2_signature(
    api_key: &str,
    api_secret: &str,
    method: &str,
    path: &str,
    query: &str,
    body: &str,
) -> (String, String, String) {
    let nonce = Uuid::new_v4().to_string();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();

    let host = "www.bitstamp.net";
    let version = "v2";
    let content_type = if body.is_empty() {
        ""
    } else {
        "application/x-www-form-urlencoded"
    };

    let string_to_sign = if body.is_empty() {
        format!(
            "BITSTAMP {}\n{}\n{}\n{}\n{}\n\n{}\n{}\n{}\n",
            api_key, method, host, path, query, nonce, timestamp, version
        )
    } else {
        format!(
            "BITSTAMP {}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            api_key, method, host, path, query, content_type,
            nonce, timestamp, version, body
        )
    };

    let mut mac = HmacSha256::new_from_slice(api_secret.as_bytes()).unwrap();
    mac.update(string_to_sign.as_bytes());
    let signature = mac.finalize().into_bytes();
    let signature_hex = hex::encode(signature).to_uppercase();

    (signature_hex, nonce, timestamp)
}
```

---

## Reference

- Official Bitstamp API Documentation: https://www.bitstamp.net/api/
- HMAC-SHA256: RFC 2104
