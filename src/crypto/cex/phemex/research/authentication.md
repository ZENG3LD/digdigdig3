# Phemex API Authentication

Complete authentication specification for V5 connector implementation.

## Authentication Method

Phemex uses HMAC SHA256 signature-based authentication for all private endpoints.

## Required HTTP Headers

All authenticated requests must include three headers:

| Header | Description | Example |
|--------|-------------|---------|
| `x-phemex-access-token` | Your API Key ID | `806066b0-f02b-4d3e-b444-76ec718e1023` |
| `x-phemex-request-expiry` | Unix timestamp (seconds) | `1575735951` |
| `x-phemex-request-signature` | HMAC SHA256 signature | `8c939f7a6e6716ab7c4240384e07c81840dacd371cdcf5051bb6b7084897470e` |

## API Key Structure

When you create an API key on Phemex, you receive:
- **API Key (id)**: Used as `x-phemex-access-token` header value
- **API Secret**: Used as HMAC key for signature generation (keep secure, never share)

## Signature Generation

### Algorithm: HMAC SHA256

The signature is computed using:
- **Key:** Your API Secret (decoded/raw bytes)
- **Message:** Concatenated string of request components

### Signature Message Format

```
URL_PATH + QUERY_STRING + EXPIRY + BODY
```

**Important:** No separators between components, just concatenate them directly.

### Component Breakdown

| Component | Description | Example |
|-----------|-------------|---------|
| URL_PATH | API endpoint path (starting with `/`) | `/orders/activeList` |
| QUERY_STRING | URL parameters (without `?`) | `ordStatus=New&symbol=BTCUSD` |
| EXPIRY | Same timestamp from header | `1575735951` |
| BODY | Request body (POST/PUT only) | `{"symbol":"BTCUSD",...}` |

### Empty Components

- **GET requests with no query:** `URL_PATH + EXPIRY`
- **GET requests with query:** `URL_PATH + QUERY_STRING + EXPIRY`
- **POST/PUT with no body:** `URL_PATH + EXPIRY`
- **POST/PUT with body:** `URL_PATH + EXPIRY + BODY`

## Expiry Timestamp

- **Format:** Unix timestamp in seconds (not milliseconds)
- **Recommended:** Current time + 60 seconds
- **Example calculation:**
  ```rust
  let expiry = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs() + 60;
  ```

## Signature Examples

### Example 1: GET Request with Query Parameters

**Request:**
```
GET /orders/activeList?ordStatus=New&ordStatus=PartiallyFilled&ordStatus=Untriggered&symbol=BTCUSD
```

**Components:**
- URL_PATH: `/orders/activeList`
- QUERY_STRING: `ordStatus=New&ordStatus=PartiallyFilled&ordStatus=Untriggered&symbol=BTCUSD`
- EXPIRY: `1575735951`
- BODY: (empty)

**Message to Sign:**
```
/orders/activeListordStatus=New&ordStatus=PartiallyFilled&ordStatus=Untriggered&symbol=BTCUSD1575735951
```

**Note:** Query string has NO `?` prefix and concatenates directly to path.

### Example 2: POST Request with Body

**Request:**
```
POST /orders
Content-Type: application/json

{
  "symbol": "BTCUSD",
  "clOrdID": "uuid-1573058952273",
  "side": "Sell",
  "priceEp": 93185000,
  "orderQty": 7,
  "ordType": "Limit",
  "reduceOnly": false,
  "timeInForce": "GoodTillCancel",
  "takeProfitEp": 0,
  "stopLossEp": 0
}
```

**Components:**
- URL_PATH: `/orders`
- QUERY_STRING: (empty)
- EXPIRY: `1575735514`
- BODY: `{"symbol":"BTCUSD","clOrdID":"uuid-1573058952273","side":"Sell","priceEp":93185000,"orderQty":7,"ordType":"Limit","reduceOnly":false,"timeInForce":"GoodTillCancel","takeProfitEp":0,"stopLossEp":0}`

**Message to Sign:**
```
/orders1575735514{"symbol":"BTCUSD","clOrdID":"uuid-1573058952273","side":"Sell","priceEp":93185000,"orderQty":7,"ordType":"Limit","reduceOnly":false,"timeInForce":"GoodTillCancel","takeProfitEp":0,"stopLossEp":0}
```

**Note:** No query string, so path connects directly to expiry, then body.

### Example 3: PUT Request with Query Parameters

**Request:**
```
PUT /orders?symbol=BTCUSD&orderID=12345678
Content-Type: application/json

{"priceEp": 94000000}
```

**Components:**
- URL_PATH: `/orders`
- QUERY_STRING: `symbol=BTCUSD&orderID=12345678`
- EXPIRY: `1575735600`
- BODY: `{"priceEp":94000000}`

**Message to Sign:**
```
/orderssymbol=BTCUSD&orderID=123456781575735600{"priceEp":94000000}
```

### Example 4: GET Request without Query

**Request:**
```
GET /spot/wallets
```

**Components:**
- URL_PATH: `/spot/wallets`
- QUERY_STRING: (empty)
- EXPIRY: `1575735800`
- BODY: (empty)

**Message to Sign:**
```
/spot/wallets1575735800
```

## Rust Implementation Pattern

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;

type HmacSha256 = Hmac<Sha256>;

fn generate_signature(
    secret: &str,
    path: &str,
    query: &str,
    expiry: u64,
    body: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Concatenate message components
    let message = format!("{}{}{}{}", path, query, expiry, body);

    // Create HMAC instance with secret
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;

    // Update with message
    mac.update(message.as_bytes());

    // Finalize and convert to hex string
    let result = mac.finalize();
    let signature = hex::encode(result.into_bytes());

    Ok(signature)
}

fn create_headers(
    api_key: &str,
    signature: &str,
    expiry: u64,
) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert("x-phemex-access-token".to_string(), api_key.to_string());
    headers.insert("x-phemex-request-expiry".to_string(), expiry.to_string());
    headers.insert("x-phemex-request-signature".to_string(), signature.to_string());
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers
}
```

## Important Notes

1. **Case Sensitivity:** The signature is case-sensitive. Ensure exact matching of paths and parameters.

2. **Query Parameter Ordering:** Parameters should be in the exact order as they appear in the URL. Some exchanges require alphabetical sorting, but Phemex uses the original order.

3. **JSON Formatting:** The body must be exactly as sent (no extra whitespace, same key ordering).

4. **Content-Type:** Always use `application/json` for request bodies.

5. **Secret Decoding:** The API secret should be used as-is (UTF-8 bytes). No Base64 decoding required for Phemex (unlike some exchanges).

6. **Signature Output:** The final signature must be lowercase hexadecimal string.

7. **Expiry Validation:** Phemex will reject requests with expired timestamps or timestamps too far in the future.

## WebSocket Authentication

For WebSocket private channels (AOP - Account Order Position):

### Authentication Message

```json
{
  "method": "user.auth",
  "params": [
    "API",
    "<api_key>",
    "<signature>",
    <expiry>
  ],
  "id": 1234
}
```

### WebSocket Signature

The WebSocket signature uses the same HMAC SHA256 algorithm but with a different message format:

**Message to Sign:**
```
<api_key><expiry>
```

**Example:**
```rust
let message = format!("{}{}", api_key, expiry);
let signature = hmac_sha256(secret, &message);
```

### WebSocket Auth Response

```json
{
  "error": null,
  "id": 1234,
  "result": {
    "status": "success"
  }
}
```

## Error Responses

### Authentication Errors

| Code | Message | Description |
|------|---------|-------------|
| 401 | Unauthorized | Invalid API key or signature |
| 403 | Forbidden | Lack of privilege for endpoint |
| - | INVALID_SIGNATURE | Signature validation failed |
| - | REQUEST_EXPIRED | Expiry timestamp is too old |
| - | INVALID_API_KEY | API key not found or disabled |

### Common Issues

1. **Invalid Signature:**
   - Check message component ordering
   - Verify no extra spaces in JSON body
   - Ensure query parameters match exactly
   - Confirm secret is correct

2. **Request Expired:**
   - System clock not synchronized
   - Expiry timestamp calculation incorrect
   - Network latency too high

3. **Forbidden:**
   - API key lacks required permissions
   - Endpoint not enabled for API key
   - IP whitelist restriction (if configured)

## Testing Authentication

### Test Endpoint

Use a simple endpoint to verify authentication:

```
GET /spot/wallets
```

This requires authentication but has minimal parameters, making it ideal for testing.

### Verification Checklist

- [ ] API key format correct (UUID format)
- [ ] Secret stored securely
- [ ] Expiry timestamp is current + 60 seconds
- [ ] Signature message concatenated correctly
- [ ] HMAC SHA256 implementation correct
- [ ] Headers included in request
- [ ] Response code 200 (not 401/403)

## Security Best Practices

1. **Never log or expose** API secrets in plain text
2. **Store secrets** in environment variables or secure vaults
3. **Use IP whitelisting** when possible
4. **Rotate API keys** periodically
5. **Set minimal permissions** on API keys (trading-only if no withdrawals needed)
6. **Monitor API usage** for unauthorized access
7. **Implement rate limiting** on client side to avoid account restrictions

## Rate Limit Headers

Authentication doesn't affect rate limits directly, but authenticated requests return rate limit headers:

```
X-RateLimit-Remaining-<GROUP>: 498
X-RateLimit-Capacity-<GROUP>: 500
X-RateLimit-Retry-After-<GROUP>: 15
```

See `rate_limits.md` for complete rate limit documentation.
