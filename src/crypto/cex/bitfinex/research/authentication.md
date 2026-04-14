# Bitfinex API v2 Authentication

## Authentication Method

Bitfinex API v2 uses **HMAC-SHA384** signature-based authentication with custom headers.

## Required Headers

All authenticated requests must include these headers:

```
Content-Type: application/json
bfx-nonce: <nonce>
bfx-apikey: <api_key>
bfx-signature: <signature>
```

## Header Details

### bfx-apikey
Your API key string as provided by Bitfinex when creating API credentials.

### bfx-nonce
An ever-increasing numeric string representing the current timestamp.

**Format**: Microseconds since Unix epoch

**Generation**: `(Date.now() * 1000).toString()` or `(epoch_ms * 1000)`

**Important Constraints**:
- Must be strictly increasing for each request
- Cannot exceed `9007199254740991` (JavaScript's MAX_SAFE_INTEGER)
- If using multiple authenticated connections, use separate API keys to avoid nonce conflicts
- Each API key maintains independent nonce tracking

### bfx-signature
HMAC-SHA384 hash of the signature payload, encoded as hexadecimal.

## Signature Generation Algorithm

### Step 1: Create the Signature String

Format: `/api/{apiPath}{nonce}{bodyJson}`

Components:
- `/api/` - Literal prefix
- `{apiPath}` - The endpoint path (e.g., `v2/auth/r/wallets`)
- `{nonce}` - The nonce value (same as used in bfx-nonce header)
- `{bodyJson}` - JSON-stringified request body (empty `{}` for endpoints with no parameters)

**Examples**:

For endpoint `/auth/r/wallets` with empty body:
```
/api/v2/auth/r/wallets1234567890000{}
```

For endpoint `/auth/w/order/submit` with order data:
```
/api/v2/auth/w/order/submit1234567890000{"type":"EXCHANGE LIMIT","symbol":"tBTCUSD","amount":"0.5","price":"10000"}
```

### Step 2: Calculate HMAC-SHA384

Use your API secret as the HMAC key and the signature string as the message.

**Pseudocode**:
```
signature_string = "/api/" + apiPath + nonce + bodyJson
hmac = HMAC_SHA384(secret_key=api_secret, message=signature_string)
signature = hex_encode(hmac)
```

### Step 3: Add to Request Header

Set `bfx-signature` header to the hexadecimal-encoded signature.

## Implementation Examples

### Python
```python
import hmac
import hashlib
import json
import time

def generate_signature(api_path, body, api_secret):
    nonce = str(int(time.time() * 1000 * 1000))  # microseconds
    body_json = json.dumps(body)

    # Create signature string
    signature_string = f"/api/{api_path}{nonce}{body_json}"

    # Calculate HMAC-SHA384
    h = hmac.new(
        api_secret.encode('utf8'),
        signature_string.encode('utf8'),
        hashlib.sha384
    )
    signature = h.hexdigest()

    return {
        'nonce': nonce,
        'signature': signature
    }

# Usage example
api_path = "v2/auth/r/wallets"
body = {}
api_secret = "your_api_secret"

auth = generate_signature(api_path, body, api_secret)

headers = {
    "Content-Type": "application/json",
    "bfx-nonce": auth['nonce'],
    "bfx-apikey": "your_api_key",
    "bfx-signature": auth['signature']
}
```

### Rust
```rust
use hmac::{Hmac, Mac};
use sha2::Sha384;
use hex;

type HmacSha384 = Hmac<Sha384>;

fn generate_signature(
    api_path: &str,
    body: &str,
    api_secret: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Generate nonce (microseconds)
    let nonce = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis() * 1000)
        .to_string();

    // Create signature string
    let signature_string = format!("/api/{}{}{}", api_path, nonce, body);

    // Calculate HMAC-SHA384
    let mut mac = HmacSha384::new_from_slice(api_secret.as_bytes())?;
    mac.update(signature_string.as_bytes());
    let result = mac.finalize();
    let signature = hex::encode(result.into_bytes());

    Ok((nonce, signature))
}

// Usage example
let api_path = "v2/auth/r/wallets";
let body = "{}";
let api_secret = "your_api_secret";

let (nonce, signature) = generate_signature(api_path, body, api_secret)?;

let mut headers = HashMap::new();
headers.insert("Content-Type", "application/json");
headers.insert("bfx-nonce", &nonce);
headers.insert("bfx-apikey", "your_api_key");
headers.insert("bfx-signature", &signature);
```

### JavaScript/Node.js
```javascript
const crypto = require('crypto');

function generateSignature(apiPath, body, apiSecret) {
    const nonce = (Date.now() * 1000).toString();
    const bodyJson = JSON.stringify(body);

    // Create signature string
    const signatureString = `/api/${apiPath}${nonce}${bodyJson}`;

    // Calculate HMAC-SHA384
    const signature = crypto
        .createHmac('sha384', apiSecret)
        .update(signatureString)
        .digest('hex');

    return { nonce, signature };
}

// Usage
const apiPath = 'v2/auth/r/wallets';
const body = {};
const apiSecret = 'your_api_secret';

const { nonce, signature } = generateSignature(apiPath, body, apiSecret);

const headers = {
    'Content-Type': 'application/json',
    'bfx-nonce': nonce,
    'bfx-apikey': 'your_api_key',
    'bfx-signature': signature
};
```

## Request Flow

1. Prepare request body as JSON object
2. Generate nonce (current time in microseconds)
3. Create signature string: `/api/{path}{nonce}{json_body}`
4. Calculate HMAC-SHA384(signature_string, api_secret)
5. Encode result as hexadecimal
6. Add headers: bfx-nonce, bfx-apikey, bfx-signature
7. Make POST request to authenticated endpoint

## Common Errors

### ERR_AUTH_NONCE_INVALID
- Nonce is not increasing
- Nonce has already been used
- System clock is incorrect

**Solution**: Ensure nonce always increases and don't reuse values.

### ERR_AUTH_SIGNATURE
- Signature calculation is incorrect
- API secret is wrong
- Signature string format is wrong

**Solution**: Verify signature string format matches exactly: `/api/{path}{nonce}{body}`

### ERR_AUTH_APIKEY
- API key is invalid or revoked
- API key doesn't exist

**Solution**: Check API key is correct and active in account settings.

## Security Best Practices

1. **Never share API secret**: Keep it secure and never commit to version control
2. **Use separate keys per client**: Prevents nonce conflicts
3. **Set IP whitelist**: Restrict API key usage to specific IP addresses
4. **Limit permissions**: Only enable required permissions (trading, withdrawal, etc.)
5. **Rotate keys regularly**: Periodically generate new API keys
6. **Monitor usage**: Check API logs for unauthorized access

## WebSocket Authentication

WebSocket connections use a similar authentication method but with different format.

See `websocket.md` for details on WebSocket authentication.

## Differences from API v1

API v2 uses different headers compared to v1:

| v1 Header | v2 Header |
|-----------|-----------|
| X-BFX-APIKEY | bfx-apikey |
| X-BFX-PAYLOAD | (not used) |
| X-BFX-SIGNATURE | bfx-signature |
| (not used) | bfx-nonce |

The signature algorithm also changed from HMAC-SHA384 of base64-encoded payload to HMAC-SHA384 of the signature string directly.
