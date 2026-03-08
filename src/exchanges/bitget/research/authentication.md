# Bitget API Authentication

## Overview

Bitget uses HMAC SHA256 signature-based authentication for private API endpoints. Public endpoints (market data) can be accessed without authentication.

## API Key Components

Three keys are required for authentication:
1. **API Key** (`ACCESS-KEY`): Public identifier
2. **Secret Key**: Used for signing (never transmitted)
3. **Passphrase** (`ACCESS-PASSPHRASE`): Additional security layer

**Security Warning:** These three keys are highly related to account security. Never disclose the Secret Key and Passphrase to anyone, as leaking any one of these keys may cause asset loss.

## Authentication Headers

All authenticated requests must include these headers:

```
ACCESS-KEY: <your_api_key>
ACCESS-SIGN: <signature>
ACCESS-TIMESTAMP: <timestamp_in_milliseconds>
ACCESS-PASSPHRASE: <your_passphrase>
Content-Type: application/json
```

## Signature Generation Process

The signature is created by encrypting a specific string with HMAC SHA256 and encoding the result in Base64.

### Step-by-Step Process

1. **Create the prehash string:**
   ```
   prehash = timestamp + method + requestPath + queryString + body
   ```

2. **Generate HMAC SHA256 signature:**
   ```
   signature = HMAC-SHA256(prehash, secretKey)
   ```

3. **Base64 encode the signature:**
   ```
   ACCESS-SIGN = Base64(signature)
   ```

### Signature Components

#### timestamp
- Same value as `ACCESS-TIMESTAMP` header
- Milliseconds since Unix Epoch
- Must be within 30 seconds of server time (recommend syncing with `/api/spot/v1/public/time`)
- Example: `1695806875837`

#### method
- HTTP method in UPPERCASE
- Examples: `GET`, `POST`, `DELETE`

#### requestPath
- Request path including `/api/...`
- Examples:
  - `/api/spot/v1/trade/orders`
  - `/api/mix/v1/order/placeOrder`

#### queryString
- Query parameters for GET requests
- Format: `?param1=value1&param2=value2`
- Include the `?` character
- For requests without query params, use empty string
- Example: `?symbol=BTCUSDT&limit=100`

#### body
- Request body as JSON string for POST requests
- Must be exact string that will be sent (no extra spaces)
- For GET requests or empty body, use empty string
- Example: `{"symbol":"BTCUSDT_SPBL","side":"buy","orderType":"limit","price":"50000.00","quantity":"0.01"}`

### Prehash String Examples

**GET Request with Query Params:**
```
1695806875837GET/api/spot/v1/market/ticker?symbol=BTCUSDT_SPBL
```

**GET Request without Query Params:**
```
1695806875837GET/api/spot/v1/account/assets
```

**POST Request:**
```
1695806875837POST/api/spot/v1/trade/orders{"symbol":"BTCUSDT_SPBL","side":"buy","orderType":"limit","price":"50000.00","quantity":"0.01"}
```

## Implementation Examples

### Python Example

```python
import hmac
import base64
import time
from hashlib import sha256

def generate_signature(secret_key, timestamp, method, request_path, query_string="", body=""):
    # Create prehash string
    if query_string and not query_string.startswith('?'):
        query_string = '?' + query_string

    prehash = str(timestamp) + method.upper() + request_path + query_string + body

    # Generate HMAC SHA256 signature
    signature = hmac.new(
        secret_key.encode('utf-8'),
        prehash.encode('utf-8'),
        sha256
    ).digest()

    # Base64 encode
    return base64.b64encode(signature).decode('utf-8')

# Example usage
api_key = "your_api_key"
secret_key = "your_secret_key"
passphrase = "your_passphrase"

timestamp = int(time.time() * 1000)
method = "POST"
request_path = "/api/spot/v1/trade/orders"
body = '{"symbol":"BTCUSDT_SPBL","side":"buy","orderType":"limit","price":"50000.00","quantity":"0.01"}'

signature = generate_signature(secret_key, timestamp, method, request_path, "", body)

headers = {
    'ACCESS-KEY': api_key,
    'ACCESS-SIGN': signature,
    'ACCESS-TIMESTAMP': str(timestamp),
    'ACCESS-PASSPHRASE': passphrase,
    'Content-Type': 'application/json'
}
```

### Rust Example

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{Engine as _, engine::general_purpose};

type HmacSha256 = Hmac<Sha256>;

fn generate_signature(
    secret_key: &str,
    timestamp: i64,
    method: &str,
    request_path: &str,
    query_string: &str,
    body: &str,
) -> String {
    // Create prehash string
    let query = if !query_string.is_empty() && !query_string.starts_with('?') {
        format!("?{}", query_string)
    } else {
        query_string.to_string()
    };

    let prehash = format!(
        "{}{}{}{}{}",
        timestamp,
        method.to_uppercase(),
        request_path,
        query,
        body
    );

    // Generate HMAC SHA256 signature
    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(prehash.as_bytes());
    let result = mac.finalize();

    // Base64 encode
    general_purpose::STANDARD.encode(result.into_bytes())
}

// Example usage
let api_key = "your_api_key";
let secret_key = "your_secret_key";
let passphrase = "your_passphrase";

let timestamp = chrono::Utc::now().timestamp_millis();
let method = "POST";
let request_path = "/api/spot/v1/trade/orders";
let body = r#"{"symbol":"BTCUSDT_SPBL","side":"buy","orderType":"limit","price":"50000.00","quantity":"0.01"}"#;

let signature = generate_signature(
    secret_key,
    timestamp,
    method,
    request_path,
    "",
    body,
);

// Build headers
let mut headers = HashMap::new();
headers.insert("ACCESS-KEY".to_string(), api_key.to_string());
headers.insert("ACCESS-SIGN".to_string(), signature);
headers.insert("ACCESS-TIMESTAMP".to_string(), timestamp.to_string());
headers.insert("ACCESS-PASSPHRASE".to_string(), passphrase.to_string());
headers.insert("Content-Type".to_string(), "application/json".to_string());
```

## RSA Signature (Alternative)

Bitget also supports RSA signature as an alternative to HMAC SHA256. This is less common and HMAC is recommended for most use cases.

### RSA Process:
1. Generate prehash string (same as HMAC)
2. Sign with RSA private key using SHA256
3. Base64 encode the signature

## WebSocket Authentication

WebSocket private channels require authentication via a login operation.

### Authentication Message

```json
{
  "op": "login",
  "args": [
    {
      "apiKey": "<your_api_key>",
      "passphrase": "<your_passphrase>",
      "timestamp": "<timestamp_in_milliseconds>",
      "sign": "<signature>"
    }
  ]
}
```

### WebSocket Signature Generation

The signature for WebSocket login is generated differently:

```
prehash = timestamp + "GET" + "/user/verify"
signature = Base64(HMAC-SHA256(prehash, secretKey))
```

**Example:**
```python
import hmac
import base64
import time
from hashlib import sha256

def generate_ws_signature(secret_key, timestamp):
    prehash = str(timestamp) + "GET" + "/user/verify"
    signature = hmac.new(
        secret_key.encode('utf-8'),
        prehash.encode('utf-8'),
        sha256
    ).digest()
    return base64.b64encode(signature).decode('utf-8')

timestamp = int(time.time() * 1000)
signature = generate_ws_signature("your_secret_key", timestamp)

login_msg = {
    "op": "login",
    "args": [{
        "apiKey": "your_api_key",
        "passphrase": "your_passphrase",
        "timestamp": str(timestamp),
        "sign": signature
    }]
}
```

## API Permissions

API keys can have different permission levels:
- **Read**: View account information, balances, orders
- **Trade**: Place and cancel orders
- **Withdraw**: Withdraw funds (requires additional security)

Configure permissions when creating the API key on Bitget website.

## IP Whitelist

For enhanced security, you can restrict API key usage to specific IP addresses. Configure this in the API management section of your Bitget account.

## Time Synchronization

**Critical:** Ensure your system clock is synchronized with Bitget servers.

- If timestamp differs from server time by more than 30 seconds, requests will be rejected
- Use `/api/spot/v1/public/time` endpoint to get server time
- Adjust your local timestamp based on server time if needed

```python
import requests
import time

def get_server_time():
    response = requests.get("https://api.bitget.com/api/spot/v1/public/time")
    data = response.json()
    return int(data['data']['serverTime'])

# Calculate time offset
server_time = get_server_time()
local_time = int(time.time() * 1000)
time_offset = server_time - local_time

# Use offset when creating timestamps
adjusted_timestamp = int(time.time() * 1000) + time_offset
```

## Common Authentication Errors

### Error 40015
**Message:** "Invalid ACCESS-TIMESTAMP"
- Timestamp is too far from server time (>30 seconds)
- Solution: Sync with server time

### Error 40016
**Message:** "Invalid ACCESS-KEY"
- API key is incorrect or doesn't exist
- Solution: Verify API key

### Error 40017
**Message:** "Invalid ACCESS-PASSPHRASE"
- Passphrase is incorrect
- Solution: Verify passphrase

### Error 40018
**Message:** "Invalid ACCESS-SIGN"
- Signature generation is incorrect
- Solution: Check prehash string construction and HMAC implementation

### Error 40019
**Message:** "Request expired"
- Timestamp is too old
- Solution: Use current timestamp

### Error 40020
**Message:** "Permission denied"
- API key doesn't have required permissions
- Solution: Update API key permissions

## Security Best Practices

1. **Never hardcode credentials** in source code
2. **Use environment variables** or secure key management systems
3. **Rotate API keys** regularly
4. **Use IP whitelist** when possible
5. **Set minimal permissions** required for your application
6. **Monitor API usage** for suspicious activity
7. **Store secret key securely** - never log or transmit it
8. **Implement proper error handling** for authentication failures

## Testing Authentication

Use a simple endpoint to test authentication:

```bash
curl -X GET "https://api.bitget.com/api/spot/v1/account/getInfo" \
  -H "ACCESS-KEY: your_api_key" \
  -H "ACCESS-SIGN: generated_signature" \
  -H "ACCESS-TIMESTAMP: 1695806875837" \
  -H "ACCESS-PASSPHRASE: your_passphrase" \
  -H "Content-Type: application/json"
```

Expected successful response:
```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": {
    "userId": "123456",
    "inviterId": "0",
    "authorities": ["trader", "spotTrader"],
    ...
  }
}
```

## Sources

- [Bitget Signature Documentation](https://www.bitget.com/api-doc/common/signature)
- [Bitget HMAC Signature Sample](https://www.bitget.com/api-doc/common/signature-samaple/hmac)
- [Bitget WebSocket API](https://www.bitget.com/api-doc/common/websocket-intro)
- [Bitget Quick Start Guide](https://www.bitget.com/api-doc/common/quick-start)
