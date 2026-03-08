# Bithumb API Authentication

## Overview

Bithumb operates two platforms with **different authentication methods**:

1. **Bithumb Korea**: JWT-based authentication with HMAC-SHA256
2. **Bithumb Pro**: Parameter signing with HMAC-SHA256

---

## Bithumb Korea Authentication

### Authentication Type
**JWT (JSON Web Token)** with HMAC-SHA256 signing

### Required Credentials
- `API Key` (access_key): Obtained from Bithumb website
- `Secret Key`: Used for signing JWT tokens

### Authentication Flow

#### 1. Generate Required Parameters

**Nonce**:
- Random UUID string for each request
- Example: `"6f5570df-d8bc-4daf-85b4-976733feb624"`
- Generate using UUID v4

**Timestamp**:
- Current time in milliseconds (13 digits)
- Example: `1712230310689`

**Query Hash** (optional, required if parameters exist):
- Hash query string parameters using SHA512
- For array parameters, use format: `key[]=value1&key[]=value2`

#### 2. Build JWT Payload

```json
{
  "access_key": "YOUR_API_KEY",
  "nonce": "uuid-v4-string",
  "timestamp": 1712230310689,
  "query_hash": "sha512_hash_of_query_string",  // optional
  "query_hash_alg": "SHA512"  // optional, default SHA512
}
```

**When to include `query_hash`**:
- Include `query_hash` and `query_hash_alg` only when request has parameters
- For requests without parameters, omit these fields

#### 3. Sign JWT Token

**Algorithm**: `HS256` (HMAC-SHA256)

**Signature Process**:
1. Create JWT with payload
2. Sign with Secret Key using HMAC-SHA256
3. Result: JWT token string

**Java Example**:
```java
Algorithm algorithm = Algorithm.HMAC256(secretKey);
String jwtToken = JWT.create()
    .withClaim("access_key", accessKey)
    .withClaim("nonce", UUID.randomUUID().toString())
    .withClaim("timestamp", System.currentTimeMillis())
    .sign(algorithm);
```

**Node.js Example**:
```javascript
const jwt = require('jsonwebtoken');
const { v4: uuidv4 } = require('uuid');

const payload = {
  access_key: 'YOUR_API_KEY',
  nonce: uuidv4(),
  timestamp: Date.now()
};

const jwtToken = jwt.sign(payload, 'YOUR_SECRET_KEY', { algorithm: 'HS256' });
```

#### 4. Add Authorization Header

```
Authorization: Bearer {jwtToken}
```

**Complete Header Example**:
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJhY2Nlc3Nfa2V5IjoiWU9VUl9BUElfS0VZIiwibm9uY2UiOiI2ZjU1NzBkZi1kOGJjLTRkYWYtODViNC05NzY3MzNmZWI2MjQiLCJ0aW1lc3RhbXAiOjE3MTIyMzAzMTA2ODl9.signature
Content-Type: application/json
```

### Authentication with Parameters

**Example**: Request with query parameters

**Parameters**:
```
order_currency=BTC&payment_currency=KRW&count=10
```

**Steps**:
1. Build query string (sorted alphabetically):
   ```
   count=10&order_currency=BTC&payment_currency=KRW
   ```

2. Hash query string with SHA512:
   ```rust
   use sha2::{Sha512, Digest};

   let query_string = "count=10&order_currency=BTC&payment_currency=KRW";
   let mut hasher = Sha512::new();
   hasher.update(query_string.as_bytes());
   let query_hash = format!("{:x}", hasher.finalize());
   ```

3. Create JWT payload:
   ```json
   {
     "access_key": "YOUR_API_KEY",
     "nonce": "uuid-v4-string",
     "timestamp": 1712230310689,
     "query_hash": "sha512_hash_result",
     "query_hash_alg": "SHA512"
   }
   ```

4. Sign and attach to request

### Request Methods

**GET Requests**: Parameters in query string
- Build query string
- Hash with SHA512
- Include in JWT payload

**POST Requests**: Parameters in request body
- Content-Type: `application/json`
- Serialize parameters to JSON string
- Hash the JSON string with SHA512
- Include in JWT payload

---

## Bithumb Pro Authentication

### Authentication Type
**Parameter Signing** with HMAC-SHA256

### Required Credentials
- `apiKey`: Public API key
- `secretKey`: Secret key for signing

### Authentication Flow

#### 1. Required Parameters

Every authenticated request must include:

| Parameter | Description | Type |
|-----------|-------------|------|
| `apiKey` | Your API key | String (Required) |
| `timestamp` | Request timestamp in milliseconds | Number (Required) |
| `signature` | HMAC-SHA256 signature | String (Required) |

Optional:
| Parameter | Description | Type |
|-----------|-------------|------|
| `msgNo` | Unique message number | String (Optional) |

#### 2. Build Signature String

**Process**:
1. Collect all request parameters (including `apiKey`, `timestamp`, etc.)
2. Sort parameters alphabetically by key
3. Join parameters with `&` in format: `key1=value1&key2=value2`
4. Sign the string with HMAC-SHA256
5. Convert signature to lowercase

**Example Parameters**:
```json
{
  "apiKey": "YOUR_API_KEY",
  "msgNo": "1234567890",
  "timestamp": 1534892332334,
  "symbol": "BTC-USDT",
  "quantity": "0.5"
}
```

**Signature String** (alphabetically sorted):
```
apiKey=YOUR_API_KEY&msgNo=1234567890&quantity=0.5&symbol=BTC-USDT&timestamp=1534892332334
```

#### 3. Generate Signature

**Algorithm**: HMAC-SHA256

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn generate_signature(params: &str, secret_key: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(params.as_bytes());
    let result = mac.finalize();
    let signature = format!("{:x}", result.into_bytes());
    signature.to_lowercase()
}
```

**Important**: Signature must be in lowercase

#### 4. Add to Request

**Method 1: In Request Body** (for POST)
```json
{
  "apiKey": "YOUR_API_KEY",
  "timestamp": 1534892332334,
  "signature": "generated_signature_in_lowercase",
  "symbol": "BTC-USDT",
  "quantity": "0.5"
}
```

**Method 2: In Query Parameters** (for GET)
```
?apiKey=YOUR_API_KEY&timestamp=1534892332334&signature=generated_signature&symbol=BTC-USDT
```

### Complete Example (Bithumb Pro)

**Request**: Create order

**Step 1: Prepare Parameters**
```rust
let mut params = HashMap::new();
params.insert("apiKey", "YOUR_API_KEY");
params.insert("timestamp", "1534892332334");
params.insert("symbol", "BTC-USDT");
params.insert("type", "limit");
params.insert("side", "buy");
params.insert("price", "50000");
params.insert("quantity", "0.5");
```

**Step 2: Sort and Join**
```rust
let mut keys: Vec<_> = params.keys().collect();
keys.sort();
let signature_string: String = keys.iter()
    .map(|k| format!("{}={}", k, params[k]))
    .collect::<Vec<_>>()
    .join("&");
// Result: apiKey=YOUR_API_KEY&price=50000&quantity=0.5&side=buy&symbol=BTC-USDT&timestamp=1534892332334&type=limit
```

**Step 3: Generate Signature**
```rust
let signature = generate_signature(&signature_string, secret_key);
params.insert("signature", &signature);
```

**Step 4: Send Request**
```rust
let response = client.post("https://global-openapi.bithumb.pro/openapi/v1/spot/placeOrder")
    .json(&params)
    .send()
    .await?;
```

---

## Bithumb Futures Authentication

### Authentication Type
**Header-based** with SHA256 HMAC

### Required Headers

| Header | Description |
|--------|-------------|
| `x-auth-key` | API key |
| `x-auth-timestamp` | Current timestamp in milliseconds |
| `x-auth-signature` | HMAC-SHA256 signature |

### Signature Generation

**MESSAGE Format**:
```
MESSAGE = timestamp + apipath
```

**Example**:
- Timestamp: `1551089460000`
- API Path: `/api/v1/user/balance`
- MESSAGE: `1551089460000/api/v1/user/balance`

**Signature** (Base64 encoded):
```bash
openssl dgst -sha256 -hmac $SECRET -binary | base64
```

**Rust Implementation**:
```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64;

type HmacSha256 = Hmac<Sha256>;

fn generate_futures_signature(timestamp: i64, api_path: &str, secret: &str) -> String {
    let message = format!("{}{}", timestamp, api_path);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(message.as_bytes());
    let result = mac.finalize();
    base64::encode(result.into_bytes())
}
```

---

## Rust Implementation Guide

### For Bithumb Korea (JWT)

```rust
use jsonwebtoken::{encode, Header, EncodingKey, Algorithm};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use sha2::{Sha512, Digest};

#[derive(Debug, Serialize, Deserialize)]
struct JwtPayload {
    access_key: String,
    nonce: String,
    timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_hash_alg: Option<String>,
}

fn generate_jwt_token(
    api_key: &str,
    secret_key: &str,
    query_string: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let nonce = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().timestamp_millis();

    let query_hash = query_string.map(|qs| {
        let mut hasher = Sha512::new();
        hasher.update(qs.as_bytes());
        format!("{:x}", hasher.finalize())
    });

    let payload = JwtPayload {
        access_key: api_key.to_string(),
        nonce,
        timestamp,
        query_hash,
        query_hash_alg: if query_string.is_some() {
            Some("SHA512".to_string())
        } else {
            None
        },
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &payload,
        &EncodingKey::from_secret(secret_key.as_bytes()),
    )?;

    Ok(token)
}

fn build_auth_header(token: &str) -> String {
    format!("Bearer {}", token)
}
```

### For Bithumb Pro (Parameter Signing)

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;

type HmacSha256 = Hmac<Sha256>;

fn sign_request(
    params: &mut HashMap<String, String>,
    api_key: &str,
    secret_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Add apiKey and timestamp
    params.insert("apiKey".to_string(), api_key.to_string());
    params.insert(
        "timestamp".to_string(),
        chrono::Utc::now().timestamp_millis().to_string(),
    );

    // Sort and join parameters
    let mut keys: Vec<_> = params.keys().cloned().collect();
    keys.sort();

    let signature_string: String = keys.iter()
        .map(|k| format!("{}={}", k, params[k]))
        .collect::<Vec<_>>()
        .join("&");

    // Generate signature
    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())?;
    mac.update(signature_string.as_bytes());
    let signature = format!("{:x}", mac.finalize().into_bytes());

    // Add signature to params (lowercase)
    params.insert("signature".to_string(), signature.to_lowercase());

    Ok(())
}
```

---

## Testing Authentication

### Test Endpoint (Bithumb Korea)
**Endpoint**: `POST /info/balance`
**Parameters**: `currency=ALL`

### Test Endpoint (Bithumb Pro)
**Endpoint**: `POST /spot/account`
**Parameters**: None

### Expected Success Response

**Bithumb Korea**:
```json
{
  "status": "0000",
  "data": {...}
}
```

**Bithumb Pro**:
```json
{
  "code": "0",
  "success": true,
  "data": {...}
}
```

---

## Common Errors

### Bithumb Korea

| Status Code | Description |
|-------------|-------------|
| `5100` | Bad Request |
| `5200` | Not Member |
| `5300` | Invalid Apikey |
| `5302` | Method Not Allowed |
| `5400` | Database Fail |
| `5500` | Invalid Parameter |
| `5600` | CUSTOM NOTICE (maintenance) |

### Bithumb Pro

| Code | Description |
|------|-------------|
| `10001` | System error |
| `10002` | Invalid parameter |
| `10003` | Illegal request |
| `10004` | Verification failed |
| `10005` | Invalid apiKey |
| `10006` | Invalid sign |
| `10007` | Illegal IP |

---

## Security Best Practices

1. **Never log or expose Secret Keys**
2. **Use environment variables** for API credentials
3. **Rotate API keys** periodically
4. **Use IP whitelisting** if available
5. **Validate timestamps** to prevent replay attacks
6. **Use HTTPS only** for all requests
7. **Generate new nonce/UUID** for each request
8. **Store signatures securely**, never in plain text
