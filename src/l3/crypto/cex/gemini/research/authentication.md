# Gemini Exchange API Authentication

Complete authentication specification for implementing V5 connector auth.rs module.

---

## Overview

Gemini uses **HMAC-SHA384** signature-based authentication for all private API endpoints (REST and WebSocket). The authentication mechanism is the same for both, with slight differences in how the payload is transmitted.

---

## Authentication Flow

### Step-by-Step Process

1. **Create JSON Payload** with request details
2. **Base64 Encode** the JSON payload
3. **Generate HMAC-SHA384 Signature** using API secret as key
4. **Convert Signature to Hexadecimal** string
5. **Add Authentication Headers** to HTTP request

---

## Required Headers

All authenticated requests must include these headers:

| Header | Description | Example |
|--------|-------------|---------|
| `X-GEMINI-APIKEY` | Your API key (public identifier) | "account-AbCdEf123456" |
| `X-GEMINI-PAYLOAD` | Base64-encoded JSON payload | "eyJyZXF1ZXN0IjoiL3YxL2..." |
| `X-GEMINI-SIGNATURE` | Hex-encoded HMAC-SHA384 signature | "a1b2c3d4e5f6..." |
| `Content-Type` | Must be `text/plain` | "text/plain" |
| `Content-Length` | Must be `0` | "0" |
| `Cache-Control` | Recommended as `no-cache` | "no-cache" |

---

## Payload Structure

### JSON Payload Format

The payload is a JSON object that **must** contain:

```json
{
  "request": "/v1/order/new",
  "nonce": 1640000000000
}
```

### Required Fields

- **`request`** (string): The API endpoint path (e.g., "/v1/balances", "/v1/order/new")
- **`nonce`** (integer/string): Unique, strictly increasing value

### Additional Fields

The payload also includes any endpoint-specific parameters:

```json
{
  "request": "/v1/order/new",
  "nonce": 1640000000000,
  "symbol": "btcusd",
  "amount": "1.5",
  "price": "50000.00",
  "side": "buy",
  "type": "exchange limit",
  "client_order_id": "my-order-123"
}
```

---

## Nonce Requirements

The nonce is critical for preventing replay attacks.

### Two Nonce Options

#### Option 1: Time-based Nonce (Recommended)

- Use Unix timestamp in **milliseconds**
- Must be within **±30 seconds** of current time
- Format: JavaScript-style millisecond timestamp

```rust
// Example: Current time in milliseconds
let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_millis() as u64;
```

**Advantages**:
- Simple to implement
- No state required between requests
- Natural ordering

**Constraints**:
- Must sync system clock
- Server rejects if outside ±30s window

#### Option 2: Sequential Nonce

- Any strictly increasing number
- Must never decrease or repeat within same API session
- No time synchronization required

```rust
// Example: Atomic counter
use std::sync::atomic::{AtomicU64, Ordering};

static NONCE: AtomicU64 = AtomicU64::new(1);

fn get_nonce() -> u64 {
    NONCE.fetch_add(1, Ordering::SeqCst)
}
```

**Advantages**:
- No clock dependency
- Works offline

**Constraints**:
- Requires state management
- Must persist across restarts for same session

### Nonce Validation

Gemini validates nonces by:
1. Checking it's greater than the last nonce received for that API key
2. (For time-based) Checking it's within ±30 seconds of server time

**Common Errors**:
- `InvalidNonce`: Nonce not increasing
- `InvalidNonce`: Time-based nonce outside allowed window

---

## Signature Generation

### Algorithm: HMAC-SHA384

The signature is created using HMAC (Hash-based Message Authentication Code) with SHA-384.

### Signature Input

The message to sign is the **Base64-encoded JSON payload** (the same value sent in `X-GEMINI-PAYLOAD`).

### Signature Steps

1. **Create JSON payload string**
   ```json
   {"request":"/v1/balances","nonce":1640000000000}
   ```

2. **Base64 encode the JSON**
   ```
   eyJyZXF1ZXN0IjoiL3YxL2JhbGFuY2VzIiwibm9uY2UiOjE2NDAwMDAwMDAwMDB9
   ```

3. **Generate HMAC-SHA384 signature**
   - Algorithm: HMAC-SHA384
   - Key: API secret (as bytes)
   - Message: Base64-encoded payload (as bytes)

4. **Convert to hexadecimal string**
   ```
   a1b2c3d4e5f6789...
   ```

### Rust Implementation

```rust
use hmac::{Hmac, Mac};
use sha2::Sha384;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

type HmacSha384 = Hmac<Sha384>;

pub fn sign_payload(payload: &str, api_secret: &str) -> Result<String, String> {
    // Step 1: Base64 encode the JSON payload
    let b64_payload = BASE64.encode(payload.as_bytes());

    // Step 2: Create HMAC-SHA384 instance with secret key
    let mut mac = HmacSha384::new_from_slice(api_secret.as_bytes())
        .map_err(|e| format!("Invalid key length: {}", e))?;

    // Step 3: Sign the base64-encoded payload
    mac.update(b64_payload.as_bytes());

    // Step 4: Finalize and convert to hex string
    let result = mac.finalize();
    let signature = hex::encode(result.into_bytes());

    Ok(signature)
}

// Helper to get the base64 payload separately
pub fn encode_payload(payload: &str) -> String {
    BASE64.encode(payload.as_bytes())
}
```

### Complete Example

```rust
use serde_json::json;

pub fn create_authenticated_request(
    api_key: &str,
    api_secret: &str,
    endpoint: &str,
    nonce: u64,
    params: serde_json::Value,
) -> HashMap<String, String> {
    // 1. Build JSON payload
    let mut payload = json!({
        "request": endpoint,
        "nonce": nonce,
    });

    // 2. Merge in additional parameters
    if let Some(obj) = params.as_object() {
        for (key, value) in obj {
            payload[key] = value.clone();
        }
    }

    let payload_str = payload.to_string();

    // 3. Base64 encode payload
    let b64_payload = BASE64.encode(payload_str.as_bytes());

    // 4. Generate signature
    let mut mac = HmacSha384::new_from_slice(api_secret.as_bytes()).unwrap();
    mac.update(b64_payload.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    // 5. Build headers
    let mut headers = HashMap::new();
    headers.insert("X-GEMINI-APIKEY".to_string(), api_key.to_string());
    headers.insert("X-GEMINI-PAYLOAD".to_string(), b64_payload);
    headers.insert("X-GEMINI-SIGNATURE".to_string(), signature);
    headers.insert("Content-Type".to_string(), "text/plain".to_string());
    headers.insert("Content-Length".to_string(), "0".to_string());
    headers.insert("Cache-Control".to_string(), "no-cache".to_string());

    headers
}
```

---

## REST API Authentication

### Request Structure

For REST API endpoints:

```
POST https://api.gemini.com/v1/balances
Headers:
  X-GEMINI-APIKEY: account-AbCdEf123456
  X-GEMINI-PAYLOAD: eyJyZXF1ZXN0IjoiL3YxL2JhbGFuY2VzIiwibm9uY2UiOjE2NDAwMDAwMDAwMDB9
  X-GEMINI-SIGNATURE: a1b2c3d4e5f6789...
  Content-Type: text/plain
  Content-Length: 0
  Cache-Control: no-cache

Body: (empty)
```

### Important Notes

1. **Body is Empty**: For POST requests, the body must be empty (Content-Length: 0)
2. **Parameters in Payload**: All parameters go in the JSON payload, NOT in query string or body
3. **Content-Type**: Must be `text/plain`, not `application/json`

### Example REST Request (Rust)

```rust
use reqwest::Client;
use serde_json::json;

async fn get_balances(
    client: &Client,
    api_key: &str,
    api_secret: &str,
    base_url: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let endpoint = "/v1/balances";
    let nonce = get_nonce();

    let payload = json!({
        "request": endpoint,
        "nonce": nonce,
    });

    let headers = create_authenticated_request(
        api_key,
        api_secret,
        endpoint,
        nonce,
        json!({}),
    );

    let url = format!("{}{}", base_url, endpoint);

    let mut req = client.post(&url);
    for (key, value) in headers {
        req = req.header(key, value);
    }

    let response = req.send().await?;
    let data = response.json().await?;

    Ok(data)
}
```

---

## WebSocket Authentication

### Connection URL

Private WebSocket endpoints use authentication in the connection request.

**Order Events WebSocket**:
```
wss://api.gemini.com/v1/order/events
```

**Sandbox**:
```
wss://api.sandbox.gemini.com/v1/order/events
```

### Authentication Process

WebSocket authentication happens **during the initial connection**, not in ongoing messages.

#### Method 1: HTTP Headers (Recommended)

Include authentication headers in the WebSocket upgrade request:

```rust
use tokio_tungstenite::{connect_async, tungstenite::handshake::client::Request};

async fn connect_order_events(
    api_key: &str,
    api_secret: &str,
) -> Result<WebSocketStream, Box<dyn std::error::Error>> {
    let endpoint = "/v1/order/events";
    let nonce = get_nonce();

    let payload = json!({
        "request": endpoint,
        "nonce": nonce,
    });

    let payload_str = payload.to_string();
    let b64_payload = BASE64.encode(payload_str.as_bytes());

    let mut mac = HmacSha384::new_from_slice(api_secret.as_bytes())?;
    mac.update(b64_payload.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    // Build WebSocket request with auth headers
    let url = "wss://api.gemini.com/v1/order/events";
    let request = Request::builder()
        .uri(url)
        .header("X-GEMINI-APIKEY", api_key)
        .header("X-GEMINI-PAYLOAD", &b64_payload)
        .header("X-GEMINI-SIGNATURE", &signature)
        .body(())?;

    let (ws_stream, _) = connect_async(request).await?;

    Ok(ws_stream)
}
```

#### Subscription Acknowledgment

After successful connection, you'll receive a `subscription_ack` message:

```json
{
  "type": "subscription_ack",
  "accountId": 123456,
  "subscriptionId": "abc-def-ghi",
  "symbolFilter": [],
  "apiSessionFilter": [],
  "eventTypeFilter": []
}
```

---

## API Key Management

### Creating API Keys

1. Log into Gemini account
2. Navigate to Settings → API
3. Create new API key
4. Assign appropriate roles:
   - **Trader**: Can view and place orders
   - **Fund Manager**: Can manage funds (deposits/withdrawals)
   - **Auditor**: Read-only access to account data

### OAuth Scopes

If using OAuth instead of API keys, required scopes:

- `balances:read`: View balances
- `orders:read`: View orders
- `orders:create`: Create/cancel orders
- `payments:read`: View payment methods
- `payments:create`: Transfer funds
- `payments:send_crypto`: Withdraw crypto
- `addresses:read`: View deposit addresses
- `addresses:create`: Create deposit addresses
- `history:read`: View transaction history

### Security Best Practices

1. **Never hardcode keys**: Use environment variables or secure vaults
2. **Rotate keys regularly**: Change API keys periodically
3. **Use minimal permissions**: Only grant required roles/scopes
4. **Separate keys per use case**: Different keys for trading vs accounting
5. **Monitor usage**: Track API key activity for anomalies
6. **Secure storage**: Encrypt API secrets at rest
7. **HTTPS only**: Never send keys over unencrypted connections

---

## Error Handling

### Authentication Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `InvalidSignature` | Signature mismatch | Check secret key, encoding, hash algorithm |
| `InvalidNonce` | Nonce not increasing | Ensure nonce increments with each request |
| `InvalidNonce` | Time window violation | Check system clock, sync time |
| `InvalidApiKey` | API key not found | Verify API key is correct and active |
| `InvalidPermissions` | Insufficient role | Check API key has required role (Trader, etc.) |
| `RateLimitExceeded` | Too many requests | Implement rate limiting, retry with backoff |
| `InvalidPayload` | Malformed JSON | Validate JSON structure before encoding |
| `SessionExpired` | OAuth token expired | Refresh OAuth token |

### Example Error Response

```json
{
  "result": "error",
  "reason": "InvalidSignature",
  "message": "Failed to verify HMAC signature"
}
```

---

## Testing Authentication

### Test Endpoint

Use the balances endpoint to test authentication:

```
POST /v1/balances
```

Successful response:
```json
[
  {
    "type": "exchange",
    "currency": "BTC",
    "amount": "1.5",
    "available": "1.0"
  }
]
```

### Sandbox Environment

Test in sandbox before production:

- **Sandbox Base URL**: `https://api.sandbox.gemini.com`
- **Sandbox WebSocket**: `wss://api.sandbox.gemini.com`
- Create separate API keys for sandbox at [sandbox.gemini.com](https://sandbox.gemini.com)

### Debugging Checklist

1. ✓ API key is correct
2. ✓ API secret is correct
3. ✓ Payload JSON is valid
4. ✓ Payload is Base64 encoded correctly
5. ✓ Nonce is increasing
6. ✓ Nonce is within ±30s (if time-based)
7. ✓ HMAC-SHA384 algorithm used (not SHA256 or SHA512)
8. ✓ Signature is hexadecimal string (lowercase)
9. ✓ All required headers present
10. ✓ Content-Type is "text/plain"
11. ✓ Content-Length is "0"
12. ✓ Request body is empty

---

## Implementation Notes for V5 Connector

### Module Structure (auth.rs)

```rust
// Expected functions for auth.rs

pub struct GeminiAuth {
    api_key: String,
    api_secret: String,
}

impl GeminiAuth {
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self { api_key, api_secret }
    }

    /// Generate nonce (millisecond timestamp)
    pub fn generate_nonce() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// Sign a payload and return authentication headers
    pub fn sign_request(
        &self,
        endpoint: &str,
        params: HashMap<String, String>,
    ) -> Result<HashMap<String, String>, ExchangeError> {
        // Implementation here
    }

    /// Sign WebSocket connection request
    pub fn sign_websocket_request(
        &self,
        endpoint: &str,
    ) -> Result<HashMap<String, String>, ExchangeError> {
        // Implementation here
    }
}
```

### Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
hmac = "0.12"
sha2 = "0.10"
base64 = "0.21"
hex = "0.4"
serde_json = "1.0"
```

---

## References

- Official Auth Docs: https://docs.gemini.com/authentication/api-key
- Private API Invocation: https://docs.gemini.com/websocket/overview/requests/private-api
- OAuth Docs: https://docs.gemini.com/rest/o-auth

---

## Summary

| Aspect | Value |
|--------|-------|
| **Algorithm** | HMAC-SHA384 |
| **Payload Format** | Base64-encoded JSON |
| **Signature Format** | Hexadecimal string (lowercase) |
| **Nonce** | Strictly increasing, ±30s window for timestamps |
| **Headers** | X-GEMINI-APIKEY, X-GEMINI-PAYLOAD, X-GEMINI-SIGNATURE |
| **Content-Type** | text/plain |
| **Request Body** | Empty (all params in payload) |
| **WebSocket Auth** | Same headers in upgrade request |
