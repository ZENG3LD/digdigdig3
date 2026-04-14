# MEXC API Authentication

## Overview

MEXC uses HMAC SHA256 signature-based authentication for all private endpoints (account, trading, wallet operations). Public market data endpoints do not require authentication.

---

## API Key Setup

### Creating API Keys

1. Create API keys in the MEXC user center
2. Set IP restrictions for security (recommended)
3. Each API Key can be linked to a maximum of 10 IP addresses
4. API Keys without linked IP addresses expire after 90 days automatically

### API Key Permissions

API keys can have different permission levels:
- `SPOT_ACCOUNT_READ`: Read account information
- `SPOT_ACCOUNT_WRITE`: Modify account (withdrawals, transfers)
- `SPOT_DEAL_READ`: Read trading information
- `SPOT_DEAL_WRITE`: Place and cancel orders

---

## Authentication Method

### Required Components

For authenticated endpoints, you must include:

1. **X-MEXC-APIKEY** header: Your API key
2. **timestamp** parameter: Current time in milliseconds
3. **signature** parameter: HMAC SHA256 signature
4. **recvWindow** parameter (optional): Request validity window

### Request Headers

```
X-MEXC-APIKEY: your_api_key_here
Content-Type: application/json
```

---

## Signature Generation

### Step 1: Build Query String

**For GET/DELETE requests:**
- Concatenate all parameters in alphabetical order with `&` separator
- Example: `symbol=BTCUSDT&side=BUY&type=LIMIT&quantity=1&price=11&recvWindow=5000&timestamp=1644489390087`

**For POST requests:**
- Parameters can be sent in query string OR request body as JSON
- If using query string: same as GET/DELETE
- If using request body: JSON string (dictionary sorting not required)

### Step 2: Create Signature Payload

Concatenate: `apiKey + timestamp + queryString`

Example payload:
```
your_api_key1644489390087symbol=BTCUSDT&side=BUY&type=LIMIT&quantity=1&price=11&recvWindow=5000&timestamp=1644489390087
```

### Step 3: Generate HMAC SHA256 Signature

Use your Secret Key to sign the payload with HMAC SHA256, then hex encode the result.

**Command Line Example (OpenSSL):**
```bash
echo -n "symbol=BTCUSDT&side=BUY&type=LIMIT&quantity=1&price=11&recvWindow=5000&timestamp=1644489390087" | \
  openssl dgst -sha256 -hmac "your_secret_key"
```

**JavaScript/Node.js Example:**
```javascript
const crypto = require('crypto');

const apiKey = 'your_api_key';
const secretKey = 'your_secret_key';
const timestamp = Date.now();
const recvWindow = 5000;

// Build query string
const queryString = `symbol=BTCUSDT&side=BUY&type=LIMIT&quantity=1&price=90000&recvWindow=${recvWindow}&timestamp=${timestamp}`;

// Create signature
const signature = crypto
  .createHmac('sha256', secretKey)
  .update(queryString)
  .digest('hex');

console.log('Signature:', signature);
```

**Python Example:**
```python
import hmac
import hashlib
import time

api_key = 'your_api_key'
secret_key = 'your_secret_key'
timestamp = int(time.time() * 1000)
recv_window = 5000

# Build query string
query_string = f'symbol=BTCUSDT&side=BUY&type=LIMIT&quantity=1&price=90000&recvWindow={recv_window}&timestamp={timestamp}'

# Create signature
signature = hmac.new(
    secret_key.encode('utf-8'),
    query_string.encode('utf-8'),
    hashlib.sha256
).hexdigest()

print(f'Signature: {signature}')
```

**Rust Example:**
```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

fn generate_signature(secret_key: &str, query_string: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(query_string.as_bytes());
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

fn main() {
    let api_key = "your_api_key";
    let secret_key = "your_secret_key";
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let recv_window = 5000;

    let query_string = format!(
        "symbol=BTCUSDT&side=BUY&type=LIMIT&quantity=1&price=90000&recvWindow={}&timestamp={}",
        recv_window, timestamp
    );

    let signature = generate_signature(secret_key, &query_string);
    println!("Signature: {}", signature);
}
```

---

## Request Timestamp and Validation

### timestamp Parameter

- **Required**: Yes, for all authenticated endpoints
- **Format**: Milliseconds since UNIX epoch
- **Purpose**: Prevents replay attacks

### recvWindow Parameter

- **Optional**: Yes
- **Default**: 5000 milliseconds
- **Maximum**: 60000 milliseconds (60 seconds)
- **Recommended**: Use a small value (5000 or less) for better security

### Server-Side Validation

The server validates requests using this logic:

```javascript
if (timestamp < (serverTime + 1000) && (serverTime - timestamp) <= recvWindow) {
  // Process request
} else {
  // Reject request with error
}
```

**Important Notes:**
1. If timestamp is more than 1 second in the future, request is rejected
2. If timestamp is older than `recvWindow` milliseconds, request is rejected
3. Ensure your system clock is synchronized (use NTP)

---

## Making Authenticated Requests

### GET Request Example

```http
GET /api/v3/account?recvWindow=5000&timestamp=1644489390087&signature=abc123...
X-MEXC-APIKEY: your_api_key
```

### POST Request Example (Query String)

```http
POST /api/v3/order?symbol=BTCUSDT&side=BUY&type=LIMIT&quantity=1&price=90000&recvWindow=5000&timestamp=1644489390087&signature=abc123...
X-MEXC-APIKEY: your_api_key
Content-Type: application/x-www-form-urlencoded
```

### POST Request Example (JSON Body)

```http
POST /api/v3/order
X-MEXC-APIKEY: your_api_key
Content-Type: application/json

{
  "symbol": "BTCUSDT",
  "side": "BUY",
  "type": "LIMIT",
  "quantity": "1",
  "price": "90000",
  "recvWindow": 5000,
  "timestamp": 1644489390087,
  "signature": "abc123..."
}
```

---

## WebSocket Authentication

### User Data Streams

User data streams (account updates, order updates) require a listen key instead of direct API key authentication.

#### Step 1: Create Listen Key (REST API)

```http
POST /api/v3/userDataStream
X-MEXC-APIKEY: your_api_key
```

**Response:**
```json
{
  "listenKey": "pqia91ma19a5s61cv6a81va65sd099v8a65a1a5s61cv6a81va65sdf19v8a65a1"
}
```

#### Step 2: Connect to WebSocket

```
wss://wbs.mexc.com/ws?listenKey=pqia91ma19a5s61cv6a81va65sd099v8a65a1a5s61cv6a81va65sdf19v8a65a1
```

#### Step 3: Keep Listen Key Alive

Send keepalive request every 30 minutes:

```http
PUT /api/v3/userDataStream?listenKey=your_listen_key
X-MEXC-APIKEY: your_api_key
```

#### Listen Key Limits

- Each UID can create maximum 60 listen keys
- Each listen key supports maximum 5 WebSocket connections
- Listen keys expire after 60 minutes without keepalive
- Single WebSocket connection valid for 24 hours maximum

---

## Error Handling

### Common Authentication Errors

**Invalid API Key:**
```json
{
  "code": 10003,
  "msg": "Invalid API key"
}
```

**Invalid Signature:**
```json
{
  "code": 10004,
  "msg": "Invalid signature"
}
```

**Invalid Timestamp:**
```json
{
  "code": 10073,
  "msg": "Invalid Request-Time"
}
```

**Timestamp Outside recvWindow:**
```json
{
  "code": 10074,
  "msg": "Timestamp for this request is outside of the recvWindow"
}
```

**Missing Required Parameter:**
```json
{
  "code": 10001,
  "msg": "Missing required parameter"
}
```

---

## Security Best Practices

### API Key Management

1. **Never share** your Secret Key
2. **Use IP whitelisting** - restrict API keys to specific IPs
3. **Set appropriate permissions** - only enable required permissions
4. **Rotate keys regularly** - especially if compromised
5. **Monitor API usage** - watch for unauthorized access

### Request Security

1. **Use HTTPS only** - never send requests over HTTP
2. **Keep recvWindow small** - 5000ms or less recommended
3. **Sync system time** - use NTP to prevent timestamp issues
4. **Store secrets securely** - use environment variables or key management systems
5. **Validate server certificates** - prevent MITM attacks

### Rate Limiting

1. **Respect rate limits** - see rate_limits.md
2. **Handle 429 errors** - implement exponential backoff
3. **Track your usage** - monitor weight consumption
4. **Avoid ban** - repeated violations result in IP ban (2 minutes to 3 days)

---

## Testing Authentication

### Test Endpoint

Use the `/api/v3/account` endpoint to test authentication:

```bash
# Generate timestamp
timestamp=$(date +%s000)

# Build query string
query="recvWindow=5000&timestamp=$timestamp"

# Generate signature (replace with your secret key)
signature=$(echo -n "$query" | openssl dgst -sha256 -hmac "your_secret_key" | awk '{print $2}')

# Make request
curl -X GET "https://api.mexc.com/api/v3/account?$query&signature=$signature" \
  -H "X-MEXC-APIKEY: your_api_key"
```

Expected success response will include your account balances and permissions.

---

## References

- MEXC uses standard HMAC SHA256 authentication similar to Binance
- All timestamps are in milliseconds (not seconds)
- Query string parameters must be alphabetically sorted for GET/DELETE
- POST requests can use either query string or JSON body
- Maximum recvWindow is 60000ms (1 minute)
