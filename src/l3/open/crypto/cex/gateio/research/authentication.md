# Gate.io API v4 Authentication Research

**Research Date**: 2026-01-21
**API Version**: v4
**Documentation**: https://www.gate.com/docs/developers/apiv4/en/

---

## Table of Contents

- [Authentication Overview](#authentication-overview)
- [Required Headers](#required-headers)
- [Signature Generation](#signature-generation)
- [Timestamp Requirements](#timestamp-requirements)
- [Request Examples](#request-examples)
- [Differences: Spot vs Futures](#differences-spot-vs-futures)
- [Error Handling](#error-handling)

---

## Authentication Overview

Gate.io API v4 uses **HMAC-SHA512** authentication for private endpoints.

All private REST requests require:
1. API Key (provided in header)
2. Signature (HMAC-SHA512 of request details)
3. Timestamp (current Unix time in seconds)

**Note**: Unlike some exchanges that use separate API keys for spot and futures, Gate.io API v4 uses **unified API keys** that work for both spot and futures trading with appropriate permissions.

---

## Required Headers

All authenticated requests **MUST** include these HTTP headers:

| Header Name | Description | Example Value | Required |
|-------------|-------------|---------------|----------|
| `KEY` | Your API key | `"67b3c5d2a9..."` | Yes |
| `SIGN` | Hexadecimal signature (HMAC-SHA512) | `"a1b2c3d4..."` | Yes |
| `Timestamp` | Unix timestamp in **seconds** | `"1737379200"` | Yes |
| `Content-Type` | Request format (for POST/PUT/PATCH) | `"application/json"` | For body requests |

**Minimal authenticated request headers**:
```http
GET /api/v4/spot/accounts HTTP/1.1
Host: api.gateio.ws
KEY: your_api_key_here
SIGN: generated_signature_here
Timestamp: 1737379200
```

---

## Signature Generation

### Algorithm: HMAC-SHA512

Gate.io uses **HMAC-SHA512** (not SHA256 like some exchanges).

### Signature String Format

The prehash string that gets signed is:
```
method + "\n" + url + "\n" + query_string + "\n" + payload_hash + "\n" + timestamp
```

### Component Details

#### 1. Method
- HTTP method in **UPPERCASE**
- Examples: `GET`, `POST`, `DELETE`, `PUT`, `PATCH`

#### 2. URL (Path)
- Request path **without** base URL, **without** query string
- Must start with `/`
- Examples:
  - `/api/v4/spot/accounts`
  - `/api/v4/spot/orders`
  - `/api/v4/futures/usdt/orders`

#### 3. Query String
- For **GET** and **DELETE** requests: include query parameters
- For **POST/PUT/PATCH** requests: usually empty string `""`
- Format: `key1=value1&key2=value2` (without leading `?`)
- Must be **URL-encoded**
- Parameters must be sorted alphabetically by key

**Example**:
```
currency_pair=BTC_USDT&status=open
```

#### 4. Payload Hash
- **HexEncode(SHA512(request_body))**
- For **GET/DELETE** requests (no body): hash of empty string
- For **POST/PUT/PATCH** requests: hash of JSON request body
- The hash must be **lowercase hexadecimal**

**Empty payload hash** (for GET/DELETE):
```
cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e
```

#### 5. Timestamp
- Same value as `Timestamp` header
- Unix timestamp in **seconds** (not milliseconds)
- String representation (e.g., `"1737379200"`)

### Signature Calculation Steps

1. **Build the prehash string**:
   ```
   prehash = method + "\n" + url + "\n" + query_string + "\n" + payload_hash + "\n" + timestamp
   ```

2. **Calculate HMAC-SHA512**:
   ```
   signature_bytes = HMAC_SHA512(api_secret, prehash_string)
   ```

3. **Convert to lowercase hexadecimal**:
   ```
   signature = hex_encode(signature_bytes).toLowerCase()
   ```

4. **Use as `SIGN` header value**:
   ```
   SIGN: {signature}
   ```

### Example Signature Generation (Pseudocode)

```python
import hashlib
import hmac
import time

api_secret = "your_api_secret"

# Request details
method = "GET"
url = "/api/v4/spot/accounts"
query_string = "currency=BTC"
timestamp = str(int(time.time()))

# Calculate payload hash (empty for GET)
payload = ""
payload_hash = hashlib.sha512(payload.encode()).hexdigest()

# Build prehash string
prehash = f"{method}\n{url}\n{query_string}\n{payload_hash}\n{timestamp}"

# Calculate signature
signature = hmac.new(
    api_secret.encode('utf-8'),
    prehash.encode('utf-8'),
    hashlib.sha512
).hexdigest()

print(f"SIGN: {signature}")
```

---

## Detailed Examples

### Example 1: GET Request (No Query Parameters)

**Request**:
```http
GET /api/v4/spot/accounts HTTP/1.1
Host: api.gateio.ws
```

**Signature calculation**:
```python
method = "GET"
url = "/api/v4/spot/accounts"
query_string = ""  # No query parameters
payload = ""
timestamp = "1737379200"

# Hash empty payload
payload_hash = hashlib.sha512(payload.encode()).hexdigest()
# = "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"

# Build prehash string
prehash = "GET\n/api/v4/spot/accounts\n\ncf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e\n1737379200"

# Calculate signature
signature = hmac.new(
    api_secret.encode(),
    prehash.encode(),
    hashlib.sha512
).hexdigest()
```

**Final headers**:
```http
KEY: your_api_key
SIGN: {calculated_signature}
Timestamp: 1737379200
```

---

### Example 2: GET Request (With Query Parameters)

**Request**:
```http
GET /api/v4/spot/orders?currency_pair=BTC_USDT&status=open HTTP/1.1
Host: api.gateio.ws
```

**Signature calculation**:
```python
method = "GET"
url = "/api/v4/spot/orders"
query_string = "currency_pair=BTC_USDT&status=open"  # Include query params
payload = ""
timestamp = "1737379200"

payload_hash = hashlib.sha512(payload.encode()).hexdigest()

prehash = "GET\n/api/v4/spot/orders\ncurrency_pair=BTC_USDT&status=open\ncf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e\n1737379200"

signature = hmac.new(api_secret.encode(), prehash.encode(), hashlib.sha512).hexdigest()
```

**Important**: Query string must match exactly what's sent in the URL.

---

### Example 3: POST Request (JSON Body)

**Request**:
```http
POST /api/v4/spot/orders HTTP/1.1
Host: api.gateio.ws
Content-Type: application/json

{
  "currency_pair": "BTC_USDT",
  "side": "buy",
  "amount": "0.01",
  "price": "48000"
}
```

**Signature calculation**:
```python
method = "POST"
url = "/api/v4/spot/orders"
query_string = ""  # No query params for POST
payload = '{"currency_pair":"BTC_USDT","side":"buy","amount":"0.01","price":"48000"}'
timestamp = "1737379200"

# Hash the JSON payload
payload_hash = hashlib.sha512(payload.encode()).hexdigest()
# = "a1b2c3d4..." (actual hash of the JSON)

prehash = f"POST\n/api/v4/spot/orders\n\n{payload_hash}\n1737379200"

signature = hmac.new(api_secret.encode(), prehash.encode(), hashlib.sha512).hexdigest()
```

**Final headers**:
```http
KEY: your_api_key
SIGN: {calculated_signature}
Timestamp: 1737379200
Content-Type: application/json
```

**Critical**: The JSON payload used in the signature **MUST** be byte-for-byte identical to what's sent in the request body. No extra spaces, no different key ordering.

---

### Example 4: DELETE Request

**Request**:
```http
DELETE /api/v4/spot/orders/123456789?currency_pair=BTC_USDT HTTP/1.1
Host: api.gateio.ws
```

**Signature calculation**:
```python
method = "DELETE"
url = "/api/v4/spot/orders/123456789"
query_string = "currency_pair=BTC_USDT"
payload = ""
timestamp = "1737379200"

payload_hash = hashlib.sha512(payload.encode()).hexdigest()

prehash = f"DELETE\n/api/v4/spot/orders/123456789\ncurrency_pair=BTC_USDT\n{payload_hash}\n1737379200"

signature = hmac.new(api_secret.encode(), prehash.encode(), hashlib.sha512).hexdigest()
```

---

## Timestamp Requirements

### Format
- **Unix timestamp in seconds** (not milliseconds)
- Must be a string in the `Timestamp` header
- Example: `"1737379200"` (not `"1737379200000"`)

### Time Synchronization
- The timestamp must be within **60 seconds** of Gate.io server time
- If the difference exceeds 60 seconds, request will be rejected with error: `INVALID_TIMESTAMP`

### Getting Server Time
**Endpoint**: `GET /api/v4/spot/time`

**Response**:
```json
{
  "server_time": 1729100692
}
```

Use this to synchronize your local time with Gate.io servers.

### Best Practices
1. Use `time.time()` (Python) or equivalent to get current Unix timestamp
2. Periodically sync with server time
3. Account for network latency (send slightly ahead of current time)
4. Cache the time offset between your system and Gate.io servers

---

## Differences: Spot vs Futures

### Good News: NO AUTHENTICATION DIFFERENCES!

**Spot and Futures use the SAME authentication mechanism**:
- Same headers (`KEY`, `SIGN`, `Timestamp`)
- Same signature algorithm (HMAC-SHA512)
- Same signature string format
- Same API keys (with appropriate permissions)

The **only** differences are:
1. **Base URL**:
   - Spot: `https://api.gateio.ws/api/v4`
   - Futures: `https://fx-api.gateio.ws/api/v4`

2. **API Key Permissions**:
   - Must enable "Spot Trading" permission for spot endpoints
   - Must enable "Futures Trading" permission for futures endpoints
   - Same key can have both permissions

3. **Endpoint Paths**:
   - Spot: `/api/v4/spot/...`
   - Futures: `/api/v4/futures/{settle}/...`

**Example**: The same authentication code works for both:
```python
# Same signature generation function for both spot and futures
def sign_request(api_secret, method, url, query_string, payload, timestamp):
    payload_hash = hashlib.sha512(payload.encode()).hexdigest()
    prehash = f"{method}\n{url}\n{query_string}\n{payload_hash}\n{timestamp}"
    return hmac.new(api_secret.encode(), prehash.encode(), hashlib.sha512).hexdigest()

# Use for spot
signature = sign_request(secret, "GET", "/api/v4/spot/accounts", "", "", timestamp)

# Use for futures (exact same function!)
signature = sign_request(secret, "GET", "/api/v4/futures/usdt/accounts", "", "", timestamp)
```

---

## Error Handling

### Authentication Errors

| Error Code | Message | Description | Solution |
|------------|---------|-------------|----------|
| `INVALID_KEY` | Invalid API key | API key not found or disabled | Check API key, regenerate if needed |
| `INVALID_SIGNATURE` | Invalid signature | Signature doesn't match | Verify signature generation logic |
| `INVALID_TIMESTAMP` | Timestamp expired | Timestamp > 60s from server time | Sync time with server |
| `IP_FORBIDDEN` | IP not whitelisted | Request from non-whitelisted IP | Add IP to whitelist in API settings |
| `PERMISSION_DENIED` | Insufficient permissions | API key lacks required permission | Enable required permissions for key |

### Common Issues

#### 1. Signature Mismatch
**Symptom**: `INVALID_SIGNATURE` error

**Common causes**:
- Query parameters not sorted alphabetically
- Query parameters not URL-encoded properly
- Extra spaces in JSON payload
- Different JSON key ordering
- Using wrong hash algorithm (SHA256 instead of SHA512)
- Payload hash not lowercase hexadecimal
- Timestamp mismatch between header and signature string

**Debug**:
```python
print(f"Prehash string:\n{prehash}\n")
print(f"Signature: {signature}")
```

#### 2. Timestamp Expired
**Symptom**: `INVALID_TIMESTAMP` error

**Solution**:
```python
# Get server time first
response = requests.get("https://api.gateio.ws/api/v4/spot/time")
server_time = response.json()["server_time"]
local_time = int(time.time())
time_offset = server_time - local_time

# Use offset for future requests
timestamp = str(int(time.time()) + time_offset)
```

#### 3. Permission Denied
**Symptom**: `PERMISSION_DENIED` error

**Solution**:
- Go to Gate.io → API Management
- Edit API key permissions
- Enable "Spot Trading" for spot endpoints
- Enable "Futures Trading" for futures endpoints
- Save changes

---

## Security Best Practices

### 1. API Key Management
- Never hardcode API keys in source code
- Use environment variables or secure key management systems
- Rotate API keys periodically
- Use different keys for different purposes (read-only, trading)

### 2. IP Whitelisting
- Enable IP whitelisting for production keys
- Use separate keys for development (without IP restrictions)

### 3. Permissions
- Enable only required permissions
- Use read-only keys for market data if possible
- Separate keys for different trading strategies

### 4. Storage
- Never commit API keys to version control
- Encrypt API keys at rest
- Use secure key storage (e.g., AWS Secrets Manager, HashiCorp Vault)

### 5. Rate Limiting
- Respect rate limits (see rate_limits.md)
- Implement exponential backoff for retries
- Cache public data when possible

---

## Implementation Checklist

- [ ] Generate HMAC-SHA512 signature (not SHA256)
- [ ] Use Unix timestamp in **seconds** (not milliseconds)
- [ ] Include all 3 required headers: `KEY`, `SIGN`, `Timestamp`
- [ ] Hash payload with SHA512 for POST requests
- [ ] Use empty payload hash for GET/DELETE requests
- [ ] Build prehash string with newlines separating components
- [ ] URL-encode query parameters
- [ ] Sort query parameters alphabetically
- [ ] Use lowercase hexadecimal for all hashes
- [ ] Ensure JSON payload in body matches hashed payload exactly
- [ ] Implement time synchronization with server
- [ ] Handle 60-second timestamp tolerance
- [ ] Test with both spot and futures endpoints
- [ ] Implement error handling for authentication failures

---

## Python Reference Implementation

```python
import hashlib
import hmac
import time
import json
import requests

class GateIOAuth:
    def __init__(self, api_key: str, api_secret: str):
        self.api_key = api_key
        self.api_secret = api_secret
        self.time_offset = 0

    def sync_time(self, base_url: str = "https://api.gateio.ws"):
        """Synchronize local time with Gate.io server time."""
        response = requests.get(f"{base_url}/api/v4/spot/time")
        server_time = response.json()["server_time"]
        local_time = int(time.time())
        self.time_offset = server_time - local_time

    def _get_timestamp(self) -> str:
        """Get current timestamp adjusted for server time."""
        return str(int(time.time()) + self.time_offset)

    def _hash_payload(self, payload: str) -> str:
        """Calculate SHA512 hash of payload."""
        return hashlib.sha512(payload.encode()).hexdigest()

    def _generate_signature(
        self,
        method: str,
        url: str,
        query_string: str,
        payload: str,
        timestamp: str
    ) -> str:
        """Generate HMAC-SHA512 signature."""
        payload_hash = self._hash_payload(payload)
        prehash = f"{method}\n{url}\n{query_string}\n{payload_hash}\n{timestamp}"

        signature = hmac.new(
            self.api_secret.encode('utf-8'),
            prehash.encode('utf-8'),
            hashlib.sha512
        ).hexdigest()

        return signature

    def sign_request(
        self,
        method: str,
        url: str,
        query_params: dict = None,
        body: dict = None
    ) -> dict:
        """Generate authentication headers for a request.

        Args:
            method: HTTP method (GET, POST, DELETE, etc.)
            url: Request path (e.g., "/api/v4/spot/accounts")
            query_params: Query parameters dict (for GET/DELETE)
            body: Request body dict (for POST/PUT/PATCH)

        Returns:
            dict: Headers including KEY, SIGN, Timestamp
        """
        timestamp = self._get_timestamp()

        # Build query string
        if query_params:
            query_string = "&".join(f"{k}={v}" for k, v in sorted(query_params.items()))
        else:
            query_string = ""

        # Build payload
        if body:
            payload = json.dumps(body, separators=(',', ':'))  # No spaces
        else:
            payload = ""

        # Generate signature
        signature = self._generate_signature(
            method.upper(),
            url,
            query_string,
            payload,
            timestamp
        )

        # Build headers
        headers = {
            "KEY": self.api_key,
            "SIGN": signature,
            "Timestamp": timestamp
        }

        if body:
            headers["Content-Type"] = "application/json"

        return headers

# Usage example
auth = GateIOAuth("your_api_key", "your_api_secret")
auth.sync_time()

# GET request
headers = auth.sign_request(
    "GET",
    "/api/v4/spot/accounts",
    query_params={"currency": "BTC"}
)

# POST request
headers = auth.sign_request(
    "POST",
    "/api/v4/spot/orders",
    body={
        "currency_pair": "BTC_USDT",
        "side": "buy",
        "amount": "0.01",
        "price": "48000"
    }
)
```

---

## Sources

- [Gate.io API Documentation - Authentication](https://www.gate.com/docs/developers/apiv4/en/)
- [Gate.io API v4 Documentation](http://www.gate.com/docs/apiv4/en/index.html)
- [GitHub - gateio/rest-v4](https://github.com/gateio/rest-v4)
- [Gate.io API Rate Limit Announcement](https://www.gate.com/announcements/article/31282)
- [Requesting to Gate API v4 using Google Apps Script](https://gist.github.com/tanaikech/d0ea117b1c0e54cf713a8027f6b2fb08)

---

**Research completed**: 2026-01-21
**Implementation note**: Signature generation is straightforward once you understand the prehash format. Key difference from other exchanges is HMAC-SHA512 (not SHA256) and the specific prehash string format with newline separators.
