# Bitget API V2 Authentication

This document describes authentication for Bitget V2 REST API.

## Authentication Overview

**Good News**: Authentication mechanism remains largely **UNCHANGED** from V1 to V2.

The same HMAC SHA256 signature algorithm is used with the same headers.

## Required Headers

All authenticated V2 endpoints require these HTTP headers:

| Header | Description | Example |
|--------|-------------|---------|
| `ACCESS-KEY` | Your API key | `bg_1234567890abcdef` |
| `ACCESS-SIGN` | HMAC SHA256 signature (Base64 encoded) | `base64(hmac_sha256(...))` |
| `ACCESS-TIMESTAMP` | Unix timestamp in milliseconds | `1695808949356` |
| `ACCESS-PASSPHRASE` | API passphrase set during key creation | `your_passphrase` |
| `Content-Type` | Content type (for POST requests) | `application/json` |
| `locale` | Language (optional) | `en-US`, `zh-CN` |

## Signature Generation

The signature is generated using HMAC SHA256 algorithm.

### Signature String Format

```
timestamp + method + requestPath + queryString + body
```

**Components**:

1. **timestamp**: Same value as `ACCESS-TIMESTAMP` header (milliseconds)
2. **method**: HTTP method in **UPPERCASE** (`GET`, `POST`, `DELETE`, etc.)
3. **requestPath**: API endpoint path (e.g., `/api/v2/spot/trade/place-order`)
4. **queryString**:
   - For GET: `?param1=value1&param2=value2` (include the `?`)
   - For POST with no query: empty string `""`
5. **body**:
   - For POST with JSON body: stringified JSON (e.g., `{"symbol":"BTCUSDT","side":"buy"}`)
   - For GET or POST without body: empty string `""`

### Signature Algorithm

```
signature = Base64(HMAC_SHA256(secretKey, signatureString))
```

Where:
- `secretKey`: Your API secret key
- `signatureString`: Constructed string as described above
- Result is Base64-encoded

## Examples

### Example 1: GET Request (Public Endpoint)

**Request**:
```
GET /api/v2/spot/market/tickers?symbol=BTCUSDT
```

**No authentication required** (public endpoint).

### Example 2: GET Request (Private Endpoint)

**Request**:
```
GET /api/v2/spot/account/assets?coin=USDT
```

**Signature Components**:
```
timestamp: 1695808949356
method: GET
requestPath: /api/v2/spot/account/assets
queryString: ?coin=USDT
body: (empty)
```

**Signature String**:
```
1695808949356GET/api/v2/spot/account/assets?coin=USDT
```

**Generate Signature** (pseudocode):
```python
import hmac
import hashlib
import base64

secret_key = "your_secret_key"
signature_string = "1695808949356GET/api/v2/spot/account/assets?coin=USDT"

signature = base64.b64encode(
    hmac.new(
        secret_key.encode('utf-8'),
        signature_string.encode('utf-8'),
        hashlib.sha256
    ).digest()
).decode('utf-8')
```

**Headers**:
```
ACCESS-KEY: your_api_key
ACCESS-SIGN: <signature>
ACCESS-TIMESTAMP: 1695808949356
ACCESS-PASSPHRASE: your_passphrase
Content-Type: application/json
```

### Example 3: POST Request (Place Order)

**Request**:
```
POST /api/v2/spot/trade/place-order
```

**Body**:
```json
{
  "symbol": "BTCUSDT",
  "side": "buy",
  "orderType": "limit",
  "force": "gtc",
  "price": "34000.00",
  "size": "0.01",
  "clientOid": "custom_123"
}
```

**Signature Components**:
```
timestamp: 1695808949356
method: POST
requestPath: /api/v2/spot/trade/place-order
queryString: (empty)
body: {"symbol":"BTCUSDT","side":"buy","orderType":"limit","force":"gtc","price":"34000.00","size":"0.01","clientOid":"custom_123"}
```

**Important**: Body must be stringified JSON **without extra spaces**.

**Signature String**:
```
1695808949356POST/api/v2/spot/trade/place-order{"symbol":"BTCUSDT","side":"buy","orderType":"limit","force":"gtc","price":"34000.00","size":"0.01","clientOid":"custom_123"}
```

**Headers**:
```
ACCESS-KEY: your_api_key
ACCESS-SIGN: <signature>
ACCESS-TIMESTAMP: 1695808949356
ACCESS-PASSPHRASE: your_passphrase
Content-Type: application/json
```

### Example 4: POST Request with Query String

**Request**:
```
POST /api/v2/spot/trade/cancel-order?symbol=BTCUSDT
```

**Body**:
```json
{
  "orderId": "1098394857234"
}
```

**Signature Components**:
```
timestamp: 1695808949356
method: POST
requestPath: /api/v2/spot/trade/cancel-order
queryString: ?symbol=BTCUSDT
body: {"orderId":"1098394857234"}
```

**Signature String**:
```
1695808949356POST/api/v2/spot/trade/cancel-order?symbol=BTCUSDT{"orderId":"1098394857234"}
```

## Rust Implementation Example

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{Engine as _, engine::general_purpose};

type HmacSha256 = Hmac<Sha256>;

pub fn generate_signature(
    secret: &str,
    timestamp: &str,
    method: &str,
    path: &str,
    query: &str,
    body: &str,
) -> String {
    // Construct signature string
    let signature_string = format!(
        "{}{}{}{}{}",
        timestamp,
        method.to_uppercase(),
        path,
        query,  // Include "?" if present, empty string otherwise
        body
    );

    // Create HMAC instance
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");

    // Update with signature string
    mac.update(signature_string.as_bytes());

    // Finalize and encode to Base64
    let result = mac.finalize();
    general_purpose::STANDARD.encode(result.into_bytes())
}

// Example usage
fn example() {
    let secret = "your_secret_key";
    let timestamp = "1695808949356";
    let method = "POST";
    let path = "/api/v2/spot/trade/place-order";
    let query = "";  // No query string for this endpoint
    let body = r#"{"symbol":"BTCUSDT","side":"buy"}"#;

    let signature = generate_signature(
        secret,
        timestamp,
        method,
        path,
        query,
        body
    );

    println!("Signature: {}", signature);
}
```

## Key Points

### ✅ Same as V1

1. **Algorithm**: HMAC SHA256 (unchanged)
2. **Headers**: Same header names and format
3. **Timestamp**: Milliseconds since Unix epoch
4. **Signature String Construction**: Same order (timestamp + method + path + query + body)
5. **Base64 Encoding**: Required for signature

### 🔍 Important Notes

1. **Method Must Be Uppercase**: `GET`, `POST`, `DELETE` (not `get`, `post`, `delete`)

2. **Query String**:
   - Include `?` prefix if present
   - URL-encoded parameter values
   - Multiple params: `?param1=value1&param2=value2`

3. **Body**:
   - Must be **exact JSON** sent in request
   - No extra whitespace or newlines
   - Empty string for GET requests

4. **Timestamp Validity**:
   - Server allows ±30 seconds tolerance
   - If timestamp differs by more than 30 seconds, request is rejected
   - Use server time endpoint `/api/v2/public/time` to sync

5. **Content-Type**:
   - Must be `application/json` for POST requests
   - Not required for GET requests

6. **Passphrase**:
   - Set when creating API key
   - Cannot be retrieved later (store securely)

## Differences from V1

**No significant differences in authentication mechanism.**

The only difference is the endpoint paths themselves (V1 vs V2), but the signature generation process is identical.

### Example: Same Auth, Different Path

**V1 Endpoint**:
```
GET /api/spot/v1/account/assets
```

**V2 Endpoint**:
```
GET /api/v2/spot/account/assets
```

**Authentication**: Exactly the same process, just different path in signature string.

## Testing Authentication

### Get Server Time (Public)

Test API connectivity without authentication:

```
GET https://api.bitget.com/api/v2/public/time
```

Response:
```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695808949356,
  "data": {
    "serverTime": "1695808949356"
  }
}
```

### Get Account Info (Private)

Test authentication with a simple authenticated endpoint:

```
GET https://api.bitget.com/api/v2/spot/account/info
```

If authentication fails, you'll receive:
```json
{
  "code": "40005",
  "msg": "Invalid signature",
  "requestTime": 1695808949356,
  "data": null
}
```

## Common Authentication Errors

| Error Code | Message | Cause | Solution |
|------------|---------|-------|----------|
| 40004 | Invalid API key | API key not found or disabled | Check API key, regenerate if needed |
| 40005 | Invalid signature | Signature mismatch | Verify signature string construction |
| 40006 | Invalid timestamp | Timestamp too old/new | Sync with server time |
| 40007 | Invalid IP | IP not whitelisted | Add IP to whitelist in API settings |
| 40008 | Passphrase error | Wrong passphrase | Verify passphrase matches API key |

## API Key Management

### Creating API Key

1. Log in to Bitget account
2. Go to API Management
3. Click "Create API Key"
4. Set permissions (Read, Trade, Withdraw)
5. Set IP whitelist (recommended)
6. Set passphrase (store securely - cannot retrieve later)
7. Complete 2FA verification
8. Save API Key and Secret Key (secret shown only once)

### Permissions

- **Read**: Access account info, balances, orders (GET endpoints)
- **Trade**: Place/cancel orders (trading endpoints)
- **Withdraw**: Withdraw funds (wallet endpoints)

**Recommendation**: Use separate API keys for different purposes, enable only required permissions.

### Security Best Practices

1. **IP Whitelist**: Always enable IP restrictions
2. **Passphrase**: Use strong, unique passphrase
3. **Permissions**: Grant minimum required permissions
4. **Storage**: Never commit API keys to source control
5. **Rotation**: Rotate keys periodically
6. **Monitoring**: Monitor API key usage for anomalies

## Rate Limits

Rate limits are enforced per API key and per IP.

**Authentication does not affect rate limit calculation** - limits are based on endpoint weight and request frequency.

See `rate_limits.md` for detailed rate limit information.

## Migration Notes

### From V1 to V2 Authentication

**No code changes needed** for authentication logic itself.

Only update:
1. Endpoint paths (`/api/spot/v1/...` → `/api/v2/spot/...`)
2. Symbol format in request/response (remove `_SPBL`, `_UMCBL` suffixes)

Your existing signature generation code will work with V2 endpoints without modification.

## Sources

- [Bitget API Signature Documentation](https://www.bitget.com/api-doc/common/signature)
- [Bitget HMAC Authentication](https://www.bitget.com/api-doc/common/signature-samaple/hmac)
- [Bitget API Quick Start](https://www.bitget.com/api-doc/common/quick-start)
- [Bitget API Introduction](https://www.bitget.com/api-doc/common/intro)
