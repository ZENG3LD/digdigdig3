# Kraken API Authentication

Kraken uses different authentication methods for Spot REST, Futures REST, and WebSocket APIs.

---

## Spot REST API Authentication

### Overview

Private Spot REST endpoints require HMAC-SHA512 signature authentication.

### Required Headers

| Header | Description |
|--------|-------------|
| `API-Key` | Your public API key (never the private key) |
| `API-Sign` | Base64-encoded HMAC-SHA512 signature |

### Required Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `nonce` | integer | Always-increasing 64-bit unsigned integer (typically UNIX timestamp in milliseconds) |
| `otp` | string | One-time password (only required if 2FA is enabled on API key) |

---

## Signature Generation Algorithm

The signature for Spot REST API is created using:

> **HMAC-SHA512 of (URI path + SHA256(nonce + POST data)) and base64 decoded secret API key**

### Step-by-Step Process

1. **Prepare the POST data string**:
   ```
   nonce=1234567890000&pair=XBTUSD&type=buy&ordertype=market&volume=0.1
   ```

2. **Concatenate nonce + POST data**:
   ```
   1234567890000nonce=1234567890000&pair=XBTUSD&type=buy&ordertype=market&volume=0.1
   ```

3. **Hash with SHA256**:
   ```rust
   let message = format!("{}{}", nonce, post_data);
   let sha256_hash = sha256(message.as_bytes());
   ```

4. **Prepend URI path to the hash digest**:
   ```rust
   let uri_path = "/0/private/AddOrder";
   let mut sign_message = uri_path.as_bytes().to_vec();
   sign_message.extend_from_slice(&sha256_hash);
   ```

5. **Base64 decode the API secret**:
   ```rust
   let secret_decoded = base64::decode(api_secret)?;
   ```

6. **Create HMAC-SHA512 signature**:
   ```rust
   use hmac::{Hmac, Mac};
   use sha2::Sha512;

   type HmacSha512 = Hmac<Sha512>;

   let mut mac = HmacSha512::new_from_slice(&secret_decoded)?;
   mac.update(&sign_message);
   let result = mac.finalize();
   let signature_bytes = result.into_bytes();
   ```

7. **Base64 encode the final signature**:
   ```rust
   let api_sign = base64::encode(&signature_bytes);
   ```

---

## URI Path Format

The URI path for private endpoints always starts with `/0/private/`:

- `/0/private/AddOrder`
- `/0/private/CancelOrder`
- `/0/private/Balance`
- `/0/private/TradeBalance`
- `/0/private/OpenOrders`
- `/0/private/QueryOrders`

---

## Nonce Requirements

### Behavior
- Must be strictly increasing per API key
- Common practice: Use UNIX timestamp in milliseconds
- Each API key maintains independent nonce sequence

### Issues to Avoid
- **Clock drift**: Can cause nonce to decrease if system time changes
- **Shared keys**: Multiple processes using same API key can cause nonce conflicts
- **Replay attacks**: Old nonce values will be rejected

### Configuration
Kraken supports optional nonce window tolerance configuration to handle minor clock drift.

---

## Rust Implementation Example

```rust
use base64;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256, Sha512};
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha512 = Hmac<Sha512>;

pub fn generate_signature(
    api_secret: &str,
    uri_path: &str,
    nonce: u64,
    post_data: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // 1. Decode the base64 API secret
    let secret_decoded = base64::decode(api_secret)?;

    // 2. Create the message to hash: nonce + POST data
    let nonce_post = format!("{}{}", nonce, post_data);

    // 3. SHA256 hash of (nonce + POST data)
    let mut hasher = Sha256::new();
    hasher.update(nonce_post.as_bytes());
    let sha256_result = hasher.finalize();

    // 4. Concatenate URI path + SHA256 hash
    let mut sign_message = uri_path.as_bytes().to_vec();
    sign_message.extend_from_slice(&sha256_result);

    // 5. HMAC-SHA512 with decoded secret
    let mut mac = HmacSha512::new_from_slice(&secret_decoded)?;
    mac.update(&sign_message);
    let signature = mac.finalize().into_bytes();

    // 6. Base64 encode the signature
    Ok(base64::encode(&signature))
}

pub fn get_nonce() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

// Example usage
fn example() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "your_api_key";
    let api_secret = "your_base64_encoded_secret";

    let nonce = get_nonce();
    let uri_path = "/0/private/Balance";
    let post_data = format!("nonce={}", nonce);

    let signature = generate_signature(api_secret, uri_path, nonce, &post_data)?;

    // Build request with headers
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.kraken.com/0/private/Balance")
        .header("API-Key", api_key)
        .header("API-Sign", signature)
        .form(&[("nonce", nonce.to_string())])
        .send()?;

    Ok(())
}
```

---

## Futures REST API Authentication

### Overview

Kraken Futures uses a different authentication mechanism from Spot.

**Important Update**: The old authentication method was phased out on October 1st, 2025. Only the new method should be used.

### Required Headers

| Header | Description |
|--------|-------------|
| `APIKey` | Your Futures API public key |
| `Authent` | Authentication signature |

### New Signature Algorithm (Post-October 2025)

The exact algorithm differs from Spot. Key points:
- Uses SHA-256 hash
- Signs decoded query string parameters
- Base64 encoding of signature

**Note**: Detailed implementation should reference the official Kraken Futures documentation as the algorithm specifics differ from Spot.

---

## WebSocket Authentication

### Spot WebSocket Authentication

#### Step 1: Get Authentication Token via REST API

**Endpoint**: `POST /0/private/GetWebSocketsToken`

**Required Parameters**:
- `nonce`: Standard nonce value

**Authentication**: Requires standard REST API authentication (API-Key + API-Sign headers)

**Response**:
```json
{
  "error": [],
  "result": {
    "token": "1Dwc4lzSwNWOAwkMdqhssNNFhs1ed606d1WcF3XfEbg",
    "expires": 900
  }
}
```

**Token Characteristics**:
- Should be used within **15 minutes** of creation
- Does **not expire** once a successful WebSocket connection and private subscription is maintained
- Valid for duration of WebSocket connection
- One token can be used for multiple private subscriptions on same connection

#### Step 2: Use Token in WebSocket Subscription

**Connection URL**: `wss://ws.kraken.com/v2`

**Subscription Message Format**:
```json
{
  "method": "subscribe",
  "params": {
    "channel": "executions",
    "token": "1Dwc4lzSwNWOAwkMdqhssNNFhs1ed606d1WcF3XfEbg",
    "snapshot": true
  }
}
```

**Private Channels Requiring Token**:
- `executions`: Trade executions
- `balances`: Account balances
- `openOrders`: Open orders (WebSocket v1: `ownTrades`, `openOrders`)

**Connection Requirements**:
- Transport Layer Security (TLS) with Server Name Indication (SNI) required
- At least one private subscription should be maintained to keep authenticated connection open
- API Key must have "WebSocket interface - On" permission

---

### Futures WebSocket Authentication

#### Step 1: Request Challenge

The Futures WebSocket uses a challenge-response authentication mechanism.

**Connection URL**: `wss://futures.kraken.com/ws/v1`

1. Client requests a challenge (returns a UUID string)
2. Client signs the challenge
3. Client sends signed challenge in subscription

#### Step 2: Sign Challenge

**Algorithm**:
1. SHA-256 hash of the challenge string
2. Base64-decode your `api_secret`
3. HMAC-SHA-512 hash using decoded secret
4. Base64-encode the result

**Rust Example**:
```rust
use base64;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256, Sha512};

type HmacSha512 = Hmac<Sha512>;

fn sign_challenge(challenge: &str, api_secret: &str) -> Result<String, Box<dyn std::error::Error>> {
    // 1. SHA-256 hash of challenge
    let mut hasher = Sha256::new();
    hasher.update(challenge.as_bytes());
    let challenge_hash = hasher.finalize();

    // 2. Base64 decode API secret
    let secret_decoded = base64::decode(api_secret)?;

    // 3. HMAC-SHA512
    let mut mac = HmacSha512::new_from_slice(&secret_decoded)?;
    mac.update(&challenge_hash);
    let signature = mac.finalize().into_bytes();

    // 4. Base64 encode
    Ok(base64::encode(&signature))
}
```

#### Step 3: Subscribe with Signed Challenge

Include both original and signed challenge in subscription:
```json
{
  "event": "subscribe",
  "feed": "fills",
  "api_key": "your_api_key",
  "original_challenge": "challenge_uuid_string",
  "signed_challenge": "base64_encoded_signature"
}
```

---

## Connection Maintenance

### WebSocket Ping/Pong

**Spot WebSocket v2**:
- Application-level ping available
- Distinct from protocol-level WebSocket ping
- **Endpoint**: Use ping method

**Futures WebSocket**:
- **Required**: Must ping at least every 60 seconds to keep connection alive
- Prevents connection timeout

---

## API Permissions

### Spot API Key Permissions

Different endpoints require different permission levels:

| Permission | Endpoints |
|------------|-----------|
| `Funds permissions - Query` | Balance, TradeBalance |
| `Orders and trades - Create & modify orders` | AddOrder, CancelOrder, EditOrder |
| `Orders and trades - Cancel & close orders` | CancelOrder, CancelAll |
| `Orders and trades - Query open orders & trades` | OpenOrders, QueryOrders (open) |
| `Orders and trades - Query closed orders & trades` | QueryOrders (closed), TradesHistory |
| `WebSocket interface - On` | Required for WebSocket connections |

### API Key Settings

When creating API keys via Kraken account settings:
- Enable specific permissions needed
- Set nonce window tolerance (optional)
- Enable/disable 2FA requirement for API calls
- Set IP whitelist (optional, recommended for security)

---

## Security Best Practices

1. **Never expose private keys**: Only send public key (API-Key header)
2. **Use HTTPS**: All REST API calls must use TLS
3. **Validate signatures**: Ensure signature generation is correct
4. **Rotate keys**: Periodically generate new API keys
5. **Limit permissions**: Only grant minimum required permissions
6. **IP whitelisting**: Restrict API key usage to known IPs
7. **Monitor usage**: Track API key activity for anomalies
8. **Secure storage**: Store API secrets encrypted, never in code

---

## Common Authentication Errors

| Error Message | Cause | Solution |
|---------------|-------|----------|
| `EAPI:Invalid key` | API key not recognized | Verify API key is correct and active |
| `EAPI:Invalid signature` | Signature calculation incorrect | Check signature algorithm implementation |
| `EAPI:Invalid nonce` | Nonce not increasing | Ensure nonce is always greater than previous |
| `EAPI:Permission denied` | API key lacks required permission | Add necessary permission to API key |
| `EAPI:Rate limit exceeded` | Too many requests | Implement rate limiting (see rate_limits.md) |

---

## Testing Authentication

### Test Endpoint: Balance Query

Simple test to verify authentication is working:

```rust
// Test authentication with Balance endpoint
async fn test_auth() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "YOUR_API_KEY";
    let api_secret = "YOUR_API_SECRET";

    let nonce = get_nonce();
    let uri_path = "/0/private/Balance";
    let post_data = format!("nonce={}", nonce);

    let signature = generate_signature(api_secret, uri_path, nonce, &post_data)?;

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.kraken.com/0/private/Balance")
        .header("API-Key", api_key)
        .header("API-Sign", signature)
        .form(&[("nonce", nonce.to_string())])
        .send()
        .await?;

    let body: serde_json::Value = response.json().await?;

    if body["error"].as_array().unwrap().is_empty() {
        println!("Authentication successful!");
        println!("Balance: {:?}", body["result"]);
    } else {
        println!("Authentication failed: {:?}", body["error"]);
    }

    Ok(())
}
```

---

## Summary

- **Spot REST**: HMAC-SHA512 with API-Key and API-Sign headers
- **Futures REST**: Different algorithm, APIKey and Authent headers
- **Spot WebSocket**: Token-based, obtained via REST endpoint
- **Futures WebSocket**: Challenge-response signature mechanism
- **Nonce**: Critical for Spot REST, must always increase
- **Permissions**: Configure via Kraken account settings
