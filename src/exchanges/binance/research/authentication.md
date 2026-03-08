# Binance API Authentication

## Overview

Binance uses HMAC-SHA256 signatures for authenticated API requests. All endpoints marked as TRADE or USER_DATA require proper authentication.

## Security Types

Binance endpoints have different security levels:

- **NONE**: Public endpoints, no authentication required
- **MARKET_DATA**: Requires API key only (no signature)
- **USER_STREAM**: Requires API key only (no signature)
- **TRADE**: Requires API key + signature
- **USER_DATA**: Requires API key + signature

## Required Headers

All authenticated requests must include:

```
X-MBX-APIKEY: <your_api_key>
```

## Authentication Algorithm: HMAC-SHA256

### Process Overview

1. **Construct Payload**: Format all parameters as `parameter=value` pairs separated by `&`
2. **Percent-Encode**: Percent-encode non-ASCII characters in the payload
3. **Generate Signature**: Use HMAC-SHA256 with your secret key to sign the payload
4. **Encode as Hex**: Convert the signature output to a hexadecimal string
5. **Append Signature**: Add the signature to the query string or request body

### Important Notes

- Signatures generated using HMAC are **not case-sensitive**
- The signature can be in uppercase or lowercase hex
- Always use your `secretKey` as the HMAC key
- The `totalParams` string is the value to be signed

## Timestamp Requirements

Every signed request **must** include a `timestamp` parameter:

- **Format**: Milliseconds since Unix epoch (or microseconds if using `X-MBX-TIME-UNIT: MICROSECOND` header)
- **Current Time**: Should reflect the current time when making the request
- **Server Validation**: The server validates that the timestamp is within acceptable range

### Timestamp Validation Formula

```
timestamp < (serverTime + 1000) && (serverTime - timestamp) <= recvWindow
```

Where:
- `serverTime`: Current server time (use `/api/v3/time` endpoint to sync)
- `recvWindow`: Request validity window in milliseconds

## RecvWindow Parameter

The `recvWindow` parameter specifies how long a request remains valid:

- **Type**: LONG (milliseconds)
- **Default**: 5000ms (if not specified)
- **Recommended**: 5000ms or less
- **Maximum**: 60000ms (1 minute)
- **Purpose**: Protects against replay attacks

**Best Practice**: Use a small recvWindow (≤5000ms) for better security.

## Signature Generation Examples

### Example 1: GET Request with Query String

**Endpoint**: `GET /api/v3/order`

**Parameters**:
```
symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559
```

**Generate Signature** (using OpenSSL):
```bash
echo -n "symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559" | openssl dgst -sha256 -hmac "YOUR_SECRET_KEY"
```

**Example Output**:
```
c8db56825ae71d6d79447849e617115f4a920fa2acdcab2b053c4b2838bd6b71
```

**Final Request**:
```
GET /api/v3/order?symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559&signature=c8db56825ae71d6d79447849e617115f4a920fa2acdcab2b053c4b2838bd6b71
```

### Example 2: POST Request with Request Body

**Endpoint**: `POST /api/v3/order`

**Request Body**:
```
symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559
```

**Headers**:
```
X-MBX-APIKEY: your_api_key
Content-Type: application/x-www-form-urlencoded
```

**Signature Calculation**: Same as GET example above

**Final Request Body**:
```
symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559&signature=c8db56825ae71d6d79447849e617115f4a920fa2acdcab2b053c4b2838bd6b71
```

### Example 3: Mixed Parameters (Query String + Body)

For POST, PUT, and DELETE endpoints, you can mix parameters between query string and request body:

**Query String**:
```
symbol=LTCBTC&timestamp=1499827319559
```

**Request Body**:
```
side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000
```

**Payload to Sign** (combine both):
```
symbol=LTCBTC&timestamp=1499827319559&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000
```

**Important**: Parameters can be sent in any order, but the signature must be calculated on the complete payload.

## Rust Implementation Pattern

### HMAC-SHA256 Signature Function

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn generate_signature(secret_key: &str, payload: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can take key of any size");

    mac.update(payload.as_bytes());

    let result = mac.finalize();
    let code_bytes = result.into_bytes();

    // Convert to hex string
    hex::encode(code_bytes)
}
```

### Building Query String

```rust
use std::collections::HashMap;

fn build_query_string(params: &HashMap<String, String>) -> String {
    let mut pairs: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();

    // Note: Order doesn't matter for signature, but consistent ordering helps debugging
    pairs.sort();
    pairs.join("&")
}
```

### Complete Signed Request Example

```rust
fn create_signed_request(
    api_key: &str,
    secret_key: &str,
    mut params: HashMap<String, String>
) -> (String, HashMap<String, String>) {
    // Add timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();

    params.insert("timestamp".to_string(), timestamp);

    // Build query string
    let query_string = build_query_string(&params);

    // Generate signature
    let signature = generate_signature(secret_key, &query_string);

    // Add signature to params
    params.insert("signature".to_string(), signature);

    // Headers
    let mut headers = HashMap::new();
    headers.insert("X-MBX-APIKEY".to_string(), api_key.to_string());
    headers.insert("Content-Type".to_string(), "application/x-www-form-urlencoded".to_string());

    (build_query_string(&params), headers)
}
```

## Percent Encoding (2026 Update)

**Important Change (Effective ~2026-01-15)**:
When calling endpoints that require signatures, **percent-encode payloads before computing signatures**.

### Characters That Need Encoding

- Non-ASCII characters must be percent-encoded
- Special characters like `+`, `=`, `&`, `?`, etc. when they appear in parameter values

### Example

Original:
```
symbol=BTC/USDT&quantity=1.5
```

Percent-encoded:
```
symbol=BTC%2FUSDT&quantity=1.5
```

## Time Synchronization

To avoid timestamp errors:

1. **Query Server Time**:
   ```
   GET /api/v3/time
   ```

2. **Calculate Offset**:
   ```rust
   let server_time = get_server_time(); // from API
   let local_time = current_time_millis();
   let time_offset = server_time - local_time;
   ```

3. **Use Offset in Requests**:
   ```rust
   let timestamp = current_time_millis() + time_offset;
   ```

## Common Authentication Errors

### Error -1021: Timestamp for this request is outside of the recvWindow

**Causes**:
- System clock is not synchronized
- Timestamp is too old
- RecvWindow is too small

**Solutions**:
- Sync with server time using `/api/v3/time`
- Increase `recvWindow` (up to 60000ms)
- Check system clock accuracy

### Error -1022: Signature for this request is not valid

**Causes**:
- Incorrect secret key
- Payload not properly formatted
- Parameters not properly encoded
- Signature not in hex format

**Solutions**:
- Verify secret key is correct
- Ensure all parameters are included in signature
- Check parameter encoding
- Verify signature is lowercase or uppercase hex

### Error -2014: API-key format invalid

**Causes**:
- Missing `X-MBX-APIKEY` header
- Incorrect API key

**Solutions**:
- Ensure header is properly set
- Verify API key is correct

## Security Best Practices

1. **Never Share Secrets**:
   - Store API keys in environment variables or secure vaults
   - Never commit keys to version control
   - Use different keys for testing and production

2. **Use Small recvWindow**:
   - Recommended: 5000ms or less
   - Reduces replay attack window

3. **IP Whitelisting**:
   - Enable IP restrictions on Binance API key settings
   - Only allow trusted IPs

4. **Key Permissions**:
   - Enable only required permissions (Spot Trading, Futures, etc.)
   - Disable withdrawal permissions if not needed

5. **Regular Rotation**:
   - Rotate API keys periodically
   - Immediately rotate if compromise is suspected

## Alternative Authentication Methods

### RSA Signatures

Binance also supports RSA signatures (RSASSA-PKCS1-v1_5):

- More secure than HMAC
- Requires RSA key pair
- Private key never sent to Binance
- Public key registered in account settings

### Ed25519 Signatures

Binance supports Ed25519 for EdDSA signatures:

- Fastest signature verification
- Smaller key size
- Modern cryptographic standard

**Note**: For V5 connector implementation, HMAC-SHA256 is recommended as the primary method due to simplicity and wide support.

## Reference Implementation

See KuCoin connector for reference:
```
v5/exchanges/kucoin/auth.rs
```

The Binance implementation should follow a similar pattern with these differences:
- Header: `X-MBX-APIKEY` (instead of `KC-API-KEY`)
- No passphrase required
- Timestamp in query parameters (not headers)
- Signature in query parameters (not headers)
