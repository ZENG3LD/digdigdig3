# Crypto.com Exchange API v1 - Authentication

## Overview

Crypto.com Exchange API v1 uses HMAC-SHA256 signature-based authentication for private endpoints. Public endpoints do not require authentication.

---

## API Key Generation

1. Login to Crypto.com Exchange
2. Navigate to **User Center > API**
3. Generate new API Key pair
4. Store `API Key` and `API Secret` securely
5. Configure IP whitelist and permissions (trading, withdrawal, etc.)

**Security Warning:** Never expose your API Secret in requests. It should only be used to generate signatures.

---

## REST API Authentication

### Required Fields for Private Endpoints

All private endpoints require these fields:

```json
{
  "id": 1,
  "method": "private/create-order",
  "api_key": "your_api_key_here",
  "params": {
    "instrument_name": "BTCUSD-PERP",
    "side": "BUY",
    "type": "LIMIT",
    "price": "50000.00",
    "quantity": "0.5"
  },
  "sig": "generated_signature_here",
  "nonce": 1587523073344
}
```

**Required Authentication Fields:**
- `api_key` - Your API Key (public identifier)
- `sig` - HMAC-SHA256 signature
- `nonce` - Unique timestamp in milliseconds

---

## Signature Generation Algorithm

### Step 1: Build Parameter String

Sort all parameter keys in ascending order (if `params` exists), then concatenate as `key + value` pairs without spaces or delimiters.

**Example Params:**
```json
{
  "instrument_name": "BTCUSD-PERP",
  "side": "BUY",
  "type": "LIMIT"
}
```

**Sorted Keys:** `instrument_name`, `side`, `type`

**Parameter String:**
```
instrument_nameBTCUSD-PERPsideBUYtypeLIMIT
```

### Step 2: Build Signature Payload

Concatenate the following in exact order:

```
method + id + api_key + parameter_string + nonce
```

**Example:**
```
private/create-order1your_api_key_hereinstrument_nameBTCUSD-PERPsideBUYtypeLIMIT1587523073344
```

### Step 3: Compute HMAC-SHA256

Use your API Secret as the cryptographic key to hash the payload.

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn generate_signature(
    method: &str,
    id: i64,
    api_key: &str,
    params_string: &str,
    nonce: i64,
    api_secret: &str,
) -> String {
    let payload = format!("{}{}{}{}{}", method, id, api_key, params_string, nonce);

    let mut mac = HmacSha256::new_from_slice(api_secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(payload.as_bytes());

    let result = mac.finalize();
    hex::encode(result.into_bytes())
}
```

### Step 4: Encode as Hex String

Convert the HMAC result to a hexadecimal string (lowercase).

---

## Complete Example

### Input Data
```
Method: private/create-order
ID: 1
API Key: api_key_example
API Secret: api_secret_example
Nonce: 1587523073344
Params: {
  "instrument_name": "BTCUSD-PERP",
  "side": "BUY",
  "type": "LIMIT",
  "price": "50000.00",
  "quantity": "0.5"
}
```

### Signature Generation Steps

**1. Sort Params and Build String:**
```
instrument_nameBTCUSD-PERPprice50000.00quantity0.5sideBUYtypeLIMIT
```

**2. Build Payload:**
```
private/create-order1api_key_exampleinstrument_nameBTCUSD-PERPprice50000.00quantity0.5sideBUYtypeLIMIT1587523073344
```

**3. Compute HMAC-SHA256:**
```rust
let signature = generate_signature(
    "private/create-order",
    1,
    "api_key_example",
    "instrument_nameBTCUSD-PERPprice50000.00quantity0.5sideBUYtypeLIMIT",
    1587523073344,
    "api_secret_example"
);
// signature = "abc123def456..." (hex encoded)
```

**4. Final Request:**
```json
{
  "id": 1,
  "method": "private/create-order",
  "api_key": "api_key_example",
  "params": {
    "instrument_name": "BTCUSD-PERP",
    "side": "BUY",
    "type": "LIMIT",
    "price": "50000.00",
    "quantity": "0.5"
  },
  "sig": "abc123def456...",
  "nonce": 1587523073344
}
```

---

## WebSocket Authentication

### Connection Flow

1. **Connect to WebSocket:**
   - User API: `wss://stream.crypto.com/exchange/v1/user`
   - **Wait 1 second** after connection before sending requests (rate limit protection)

2. **Send Authentication Request:**
   Invoke `public/auth` **once per session** with signature.

3. **Session Authentication:**
   Once authenticated, all subsequent user-specific subscriptions/commands do not require `api_key` or `sig`.

### Authentication Message

```json
{
  "id": 1,
  "method": "public/auth",
  "api_key": "your_api_key_here",
  "sig": "generated_signature_here",
  "nonce": 1587523073344
}
```

### Signature Generation for WebSocket Auth

**Payload:**
```
public/auth1your_api_key_here1587523073344
```

**Note:** No params for auth method, so parameter string is empty.

**Rust Example:**
```rust
fn generate_ws_auth_signature(
    id: i64,
    api_key: &str,
    nonce: i64,
    api_secret: &str,
) -> String {
    let payload = format!("public/auth{}{}{}", id, api_key, nonce);

    let mut mac = HmacSha256::new_from_slice(api_secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(payload.as_bytes());

    hex::encode(mac.finalize().into_bytes())
}
```

### Authentication Response

```json
{
  "id": 1,
  "method": "public/auth",
  "code": 0
}
```

**Success:** `code: 0`
**Failure:** `code` will be non-zero with error message

---

## Nonce Management

### Requirements
- **Unique:** Each request must have a unique nonce
- **Incrementing:** Nonces should increase over time
- **Format:** Integer (typically milliseconds since Unix epoch)

### Best Practices

```rust
use std::time::{SystemTime, UNIX_EPOCH};

fn generate_nonce() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64
}
```

### Anti-Pattern
```rust
// DON'T: Reusing nonces
let nonce = 123456789;
request_1.nonce = nonce;
request_2.nonce = nonce; // ERROR: Duplicate nonce
```

---

## Error Handling

### Common Authentication Errors

| Code | Message | Cause | Solution |
|------|---------|-------|----------|
| 10003 | INVALID_SIGNATURE | Signature mismatch | Verify signature algorithm |
| 10004 | INVALID_NONCE | Duplicate/old nonce | Use unique increasing nonce |
| 10005 | INVALID_API_KEY | API key not found | Check API key validity |
| 10006 | IP_NOT_WHITELISTED | Request from non-whitelisted IP | Add IP to whitelist |
| 10008 | PERMISSION_DENIED | Insufficient permissions | Enable required permissions |

### Error Response Format

```json
{
  "id": 1,
  "method": "private/create-order",
  "code": 10003,
  "message": "INVALID_SIGNATURE",
  "original": "..."
}
```

---

## Security Best Practices

### 1. Secure Key Storage
```rust
// Use environment variables
let api_key = std::env::var("CRYPTO_COM_API_KEY")
    .expect("CRYPTO_COM_API_KEY not set");
let api_secret = std::env::var("CRYPTO_COM_API_SECRET")
    .expect("CRYPTO_COM_API_SECRET not set");
```

### 2. Never Log Secrets
```rust
// DON'T
println!("API Secret: {}", api_secret);

// DO
println!("Signature: {}", signature);
```

### 3. Use HTTPS/WSS
- Always use production URLs (HTTPS/WSS)
- Validate SSL certificates
- Never disable certificate verification

### 4. IP Whitelisting
- Whitelist only necessary IPs
- Use static IPs for production
- Regularly audit whitelist

### 5. Permission Management
- Grant minimum required permissions
- Separate API keys for different purposes
- Never enable withdrawal permissions unless necessary

---

## Request ID Management

The `id` field in requests:
- **Purpose:** Match requests with responses
- **Type:** Integer
- **Uniqueness:** Should be unique per request (can use incrementing counter)

```rust
struct ApiClient {
    request_counter: std::sync::atomic::AtomicI64,
}

impl ApiClient {
    fn next_request_id(&self) -> i64 {
        self.request_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}
```

---

## Rate Limit Considerations

Authentication does not bypass rate limits:
- Private endpoints have stricter limits than public
- Failed authentication attempts count toward rate limits
- See `rate_limits.md` for detailed limits

---

## Testing Authentication

### Sandbox Environment

**UAT URLs:**
- REST: `https://uat-api.3ona.co/exchange/v1/{method}`
- WS User: `wss://uat-stream.3ona.co/exchange/v1/user`

**Test Procedure:**
1. Generate sandbox API keys from UAT environment
2. Verify signature generation with known test vectors
3. Test nonce uniqueness
4. Test error handling with invalid signatures

### Simple Test Request

```rust
// Test with private/user-balance (no params)
let method = "private/user-balance";
let id = 1;
let nonce = generate_nonce();
let params_string = ""; // No params

let signature = generate_signature(method, id, &api_key, params_string, nonce, &api_secret);

let request = json!({
    "id": id,
    "method": method,
    "api_key": api_key,
    "params": {},
    "sig": signature,
    "nonce": nonce
});
```

---

## Troubleshooting

### Signature Mismatch

**Common Causes:**
1. Incorrect parameter sorting
2. Missing/extra spaces in payload
3. Wrong encoding (must be lowercase hex)
4. Incorrect nonce value
5. Using wrong API secret

**Debugging Steps:**
1. Print payload before hashing
2. Verify parameter string construction
3. Check nonce matches in signature and request
4. Validate API secret
5. Compare with working example

### Example Debug Output

```rust
let payload = format!("{}{}{}{}{}", method, id, api_key, params_string, nonce);
println!("Payload: {}", payload);
println!("Signature: {}", signature);
```

Expected payload format:
```
private/create-order1api_key_exampleinstrument_nameBTCUSD-PERP...1587523073344
```

---

## Implementation Checklist

- [ ] API key/secret loaded from secure storage
- [ ] HMAC-SHA256 implementation tested
- [ ] Parameter sorting implemented correctly
- [ ] Payload concatenation follows exact format
- [ ] Hex encoding (lowercase) verified
- [ ] Nonce generation using milliseconds
- [ ] Request ID management implemented
- [ ] WebSocket auth flow tested
- [ ] Error handling for auth failures
- [ ] Rate limit awareness
- [ ] No secrets logged or exposed
- [ ] Tested in sandbox environment
