# BingX API Authentication

Complete authentication documentation for BingX API V5 connector implementation.

---

## Overview

BingX uses HMAC SHA256 signature-based authentication for private endpoints. All authenticated requests require:

1. **API Key** - Included in request headers
2. **Timestamp** - Request timestamp in milliseconds
3. **Signature** - HMAC SHA256 hash of request parameters

---

## API Key Generation

### Creating API Keys

1. Log in to BingX account
2. Navigate to API Management: `https://bingx.com/en-us/account/api/`
3. Create new API key
4. Save both:
   - **API Key** - Public identifier
   - **Secret Key** - Private signing key (never share or expose)

### API Key Permissions

Configure permissions when creating API keys:
- **Read** - Query account data, orders, positions
- **Trade** - Place and cancel orders
- **Withdraw** - Withdraw funds (not recommended for trading bots)

---

## Authentication Requirements

### Required Headers

All authenticated requests must include:

```
X-BX-APIKEY: <your_api_key>
```

### Required Parameters

All authenticated requests must include these parameters:

1. **timestamp** (required)
   - Current timestamp in milliseconds
   - Must be within 5000ms (5 seconds) of server time
   - Example: `1649404670162`

2. **signature** (required)
   - HMAC SHA256 hash of request parameters
   - Computed using secret key

3. **recvWindow** (optional)
   - Request validity window in milliseconds
   - Default: 5000ms
   - Maximum: 60000ms (60 seconds)
   - Defines how long the request remains valid

---

## Signature Generation

### Algorithm: HMAC SHA256

BingX uses HMAC SHA256 encryption to generate signatures from assembled request parameters.

### Step-by-Step Process

#### 1. Assemble Request Parameters

Concatenate all request parameters (excluding `signature`) in **query string format**:

**Query String Rules:**
- Parameters in alphabetical order (optional but recommended for consistency)
- Format: `key1=value1&key2=value2&key3=value3`
- Include `timestamp` parameter
- Do NOT include `signature` parameter
- URL encode parameter values if they contain special characters

**Example for Spot Order:**
```
quoteOrderQty=20&side=BUY&symbol=ETH-USDT&timestamp=1649404670162&type=MARKET
```

**Example for Swap Order:**
```
positionSide=LONG&quantity=0.1&side=BUY&symbol=BTC-USDT&timestamp=1649404670162&type=MARKET
```

#### 2. Generate HMAC SHA256 Signature

Use your **Secret Key** to sign the assembled parameter string:

**Pseudo-code:**
```
signature = HMAC_SHA256(secret_key, parameter_string)
signature_hex = hex_encode(signature)
```

**OpenSSL Command Line Example:**
```bash
echo -n "quoteOrderQty=20&side=BUY&symbol=ETH-USDT&timestamp=1649404670162&type=MARKET" | \
  openssl dgst -sha256 -hmac "YOUR_SECRET_KEY" -hex
```

Output:
```
(stdin)= a1b2c3d4e5f6...
```

#### 3. Include Signature in Request

Add the signature as a parameter:

**For GET/DELETE requests:**
```
GET /openApi/spot/v1/trade/order?symbol=BTC-USDT&orderId=123456&timestamp=1649404670162&signature=a1b2c3d4e5f6...
```

**For POST requests:**
```
POST /openApi/swap/v2/trade/order
Content-Type: application/x-www-form-urlencoded

symbol=BTC-USDT&side=BUY&type=MARKET&quantity=0.1&timestamp=1649404670162&signature=a1b2c3d4e5f6...
```

---

## Implementation Examples

### Rust Implementation

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

/// Generate timestamp in milliseconds
fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Generate HMAC SHA256 signature
fn generate_signature(secret_key: &str, params: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(params.as_bytes());
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

/// Build query string from parameters
fn build_query_string(params: &HashMap<String, String>) -> String {
    let mut pairs: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();
    pairs.sort(); // Optional: sort for consistency
    pairs.join("&")
}

/// Sign a request
fn sign_request(
    secret_key: &str,
    params: &mut HashMap<String, String>,
) -> String {
    // Add timestamp
    let timestamp = get_timestamp();
    params.insert("timestamp".to_string(), timestamp.to_string());

    // Build parameter string
    let param_string = build_query_string(params);

    // Generate signature
    let signature = generate_signature(secret_key, &param_string);

    // Add signature to params
    params.insert("signature".to_string(), signature.clone());

    signature
}
```

### Example Usage

```rust
use std::collections::HashMap;

fn main() {
    let api_key = "your_api_key";
    let secret_key = "your_secret_key";

    // Prepare order parameters
    let mut params = HashMap::new();
    params.insert("symbol".to_string(), "BTC-USDT".to_string());
    params.insert("side".to_string(), "BUY".to_string());
    params.insert("type".to_string(), "MARKET".to_string());
    params.insert("quantity".to_string(), "0.1".to_string());

    // Sign the request
    let signature = sign_request(secret_key, &mut params);

    // Build request URL
    let query_string = build_query_string(&params);
    let url = format!("https://open-api.bingx.com/openApi/swap/v2/trade/order?{}", query_string);

    // Make request with API key header
    // headers.insert("X-BX-APIKEY", api_key);
}
```

---

## Authentication Flow

### 1. Public Endpoints (No Authentication)

```
GET https://open-api.bingx.com/openApi/spot/v1/market/depth?symbol=BTC-USDT&limit=20
```

**Headers:** None required

### 2. Private Endpoints (Authenticated)

```
POST https://open-api.bingx.com/openApi/swap/v2/trade/order
```

**Headers:**
```
X-BX-APIKEY: your_api_key
Content-Type: application/x-www-form-urlencoded
```

**Body:**
```
symbol=BTC-USDT&side=BUY&type=MARKET&quantity=0.1&timestamp=1649404670162&signature=a1b2c3d4...
```

**Process:**
1. Collect all parameters: `symbol`, `side`, `type`, `quantity`
2. Add `timestamp`: current time in milliseconds
3. Build parameter string: `quantity=0.1&side=BUY&symbol=BTC-USDT&timestamp=1649404670162&type=MARKET`
4. Generate signature using secret key
5. Add `signature` parameter
6. Add `X-BX-APIKEY` header
7. Send request

---

## Timestamp Validation

### Server Time Synchronization

**Problem:** If local clock is out of sync with server, requests will be rejected.

**Solution:** Query server time and adjust local timestamp.

**Server Time Endpoint:**
```
GET /openApi/swap/v2/server/time
```

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": {
    "serverTime": 1649404670162
  }
}
```

### Handling Time Differences

```rust
fn get_server_time_offset(client: &reqwest::Client) -> i64 {
    let server_time = fetch_server_time(client); // in milliseconds
    let local_time = get_timestamp(); // in milliseconds
    server_time as i64 - local_time as i64
}

fn get_adjusted_timestamp(offset: i64) -> u64 {
    let local = get_timestamp();
    (local as i64 + offset) as u64
}
```

### Request Window

The `timestamp` parameter must satisfy:

```
server_time - timestamp <= recvWindow (default 5000ms)
```

If timestamp is too old (more than `recvWindow` milliseconds), request is rejected:

**Error Response:**
```json
{
  "code": 100400,
  "msg": "Timestamp for this request is outside of the recvWindow."
}
```

---

## Common Authentication Errors

### 1. Invalid Signature (100401)

**Error:**
```json
{
  "code": 100401,
  "msg": "AUTHENTICATION_FAIL"
}
```

**Causes:**
- Incorrect secret key
- Wrong parameter string format
- Missing parameters in signature calculation
- Parameters in request don't match signed parameters

**Fix:**
- Verify secret key is correct
- Ensure all parameters (except `signature`) are included in signing
- Check parameter order and format
- Debug: print parameter string before signing

### 2. Timestamp Out of Window (100400)

**Error:**
```json
{
  "code": 100400,
  "msg": "Timestamp for this request is outside of the recvWindow."
}
```

**Causes:**
- Local clock out of sync
- Network latency too high
- Request retry with old timestamp

**Fix:**
- Sync local clock with server time
- Increase `recvWindow` (up to 60000ms)
- Generate new timestamp for each request

### 3. Missing API Key (100401)

**Error:**
```json
{
  "code": 100401,
  "msg": "API-key header is required"
}
```

**Cause:**
- `X-BX-APIKEY` header not included

**Fix:**
- Add header: `X-BX-APIKEY: your_api_key`

### 4. Invalid API Key (100401)

**Error:**
```json
{
  "code": 100401,
  "msg": "Invalid API-key"
}
```

**Causes:**
- Wrong API key
- API key deleted or disabled
- IP whitelist restriction (if enabled)

**Fix:**
- Verify API key is correct
- Check API key status in account settings
- Add IP to whitelist if IP restriction enabled

### 5. Insufficient Permissions (100403)

**Error:**
```json
{
  "code": 100403,
  "msg": "AUTHORIZATION_FAIL"
}
```

**Cause:**
- API key lacks required permissions

**Fix:**
- Enable trading permission for API key
- Recreate API key with correct permissions

---

## Security Best Practices

### 1. Protect Secret Keys

- **Never** commit secret keys to version control
- Store in environment variables or secure key management
- Use separate API keys for development/production
- Rotate keys periodically

### 2. Minimize Permissions

- Only enable required permissions
- Avoid withdraw permission for trading bots
- Use read-only keys for monitoring

### 3. IP Whitelisting

- Enable IP whitelist in API settings
- Only allow trusted IP addresses
- Update whitelist when IP changes

### 4. Monitor API Usage

- Track all API key usage
- Set up alerts for suspicious activity
- Regularly review API access logs

### 5. Handle Credentials Securely

```rust
// Good: Load from environment
let api_key = std::env::var("BINGX_API_KEY").expect("API key not set");
let secret_key = std::env::var("BINGX_SECRET_KEY").expect("Secret key not set");

// Bad: Hardcoded credentials
// let api_key = "my_api_key_12345"; // NEVER DO THIS
```

---

## WebSocket Authentication

### Listen Key Method

BingX uses listen keys for WebSocket user data streams.

#### 1. Generate Listen Key (REST)

**Spot:**
```
POST /openApi/spot/v1/user/listen-key
Headers: X-BX-APIKEY: your_api_key
```

**Swap:**
```
POST /openApi/swap/v2/user/listen-key
Headers: X-BX-APIKEY: your_api_key
```

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": {
    "listenKey": "pqia91ma19a5s61cv6a81va65sdf19v8a65a1a5s61cv6a81va65sdf19v8a65a1"
  }
}
```

#### 2. Connect to WebSocket

```
wss://open-api-ws.bingx.com/market?listenKey=pqia91ma19a5s61cv6a81va65sdf19v8a65a1a5s61cv6a81va65sdf19v8a65a1
```

#### 3. Maintain Listen Key

Listen keys expire after 1 hour. Extend validity every 30 minutes:

**Extend (REST):**
```
PUT /openApi/spot/v1/user/listen-key
Headers: X-BX-APIKEY: your_api_key
```

**Implementation:**
```rust
// Extend listen key every 30 minutes
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(30 * 60));
    loop {
        interval.tick().await;
        extend_listen_key(&api_key).await;
    }
});
```

#### 4. Delete Listen Key

When closing connection:
```
DELETE /openApi/spot/v1/user/listen-key
Headers: X-BX-APIKEY: your_api_key
```

---

## Testing Authentication

### 1. Test Connectivity

```
GET https://open-api.bingx.com/openApi/spot/v1/common/symbols
```

Should work without authentication.

### 2. Test API Key

```
GET https://open-api.bingx.com/openApi/spot/v1/account/balance?timestamp=<current_ms>&signature=<signed>
Headers: X-BX-APIKEY: your_api_key
```

Should return account balance if authenticated correctly.

### 3. Verify Signature

Print parameter string and signature before sending:

```rust
println!("Param string: {}", param_string);
println!("Signature: {}", signature);
```

Compare with expected values.

### 4. Check Server Response

Handle authentication errors gracefully:

```rust
match response.status() {
    StatusCode::OK => { /* Success */ },
    StatusCode::UNAUTHORIZED => {
        eprintln!("Authentication failed: check API key and signature");
    },
    StatusCode::FORBIDDEN => {
        eprintln!("Forbidden: check API key permissions");
    },
    _ => { /* Other errors */ }
}
```

---

## Reference Implementation

### Complete Signing Function

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct BingXAuth {
    api_key: String,
    secret_key: String,
    time_offset: i64, // milliseconds
}

impl BingXAuth {
    pub fn new(api_key: String, secret_key: String) -> Self {
        Self {
            api_key,
            secret_key,
            time_offset: 0,
        }
    }

    pub fn set_time_offset(&mut self, offset: i64) {
        self.time_offset = offset;
    }

    fn get_timestamp(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        (now as i64 + self.time_offset) as u64
    }

    fn build_query_string(&self, params: &HashMap<String, String>) -> String {
        let mut pairs: Vec<String> = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        pairs.sort();
        pairs.join("&")
    }

    fn generate_signature(&self, param_string: &str) -> String {
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(param_string.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    pub fn sign_request(&self, params: &mut HashMap<String, String>) -> String {
        // Add timestamp
        params.insert("timestamp".to_string(), self.get_timestamp().to_string());

        // Build parameter string (without signature)
        let param_string = self.build_query_string(params);

        // Generate signature
        let signature = self.generate_signature(&param_string);

        // Add signature to params
        params.insert("signature".to_string(), signature.clone());

        signature
    }

    pub fn get_api_key(&self) -> &str {
        &self.api_key
    }
}
```

---

## Sources

- [BingX API Docs - Authentication](https://bingx-api.github.io/docs/#/swapV2/authentication.html)
- [BingX API GitHub - Standard Contract](https://github.com/BingX-API/BingX-Standard-Contract-doc/blob/main/REST%20API.md)
- [Node.js Signature Generation](https://community.latenode.com/t/what-is-the-method-to-generate-a-bingx-api-signature-using-node-js/513)
- [CCXT BingX Issues - Signature](https://github.com/ccxt/ccxt/issues/24883)
- [BingX WebSocket User Data](https://hexdocs.pm/bingex/Bingex.User.html)
